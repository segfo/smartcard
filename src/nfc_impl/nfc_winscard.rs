// Windows SCardAPIによる実装を記述する
use crate::pc_sc_standard::*;
use crate::smart_card;
use crate::smart_card::*;
use winapi::shared::winerror::*;
use winapi::um::{winscard::*, winsmcrd::*};
const INFINITE: u32 = 0xffff_ffff;

#[derive(Clone, Copy)]
struct ProtocolTypeSet(ProtocolType, u32);
impl ProtocolTypeSet {
    fn get_protocol(&self) -> SCARD_IO_REQUEST {
        SCARD_IO_REQUEST {
            dwProtocol: self.1,
            cbPciLength: std::mem::size_of::<SCARD_IO_REQUEST>() as u32,
        }
    }
}

pub struct WinScardNFC {
    ctx: SCARDCONTEXT,
    h_scard: SCARDHANDLE,
    protocol: ProtocolTypeSet,
    atr: AnswerToReset,
}

// WinScardの実装
impl WinScardNFC {
    pub fn new() -> Result<Self, SmartcardError> {
        let ctx = unsafe {
            let mut ctx: SCARDCONTEXT = 0;
            let ret = SCardEstablishContext(
                SCARD_SCOPE_USER,
                std::ptr::null(),
                std::ptr::null(),
                &mut ctx,
            );
            if ret != SCARD_S_SUCCESS {
                return Err(SmartcardError::new(SmartcardErrorKind::ResMgrCtxInit));
            }
            ctx
        };
        Ok(WinScardNFC {
            ctx: ctx,
            h_scard: 0,
            atr: AnswerToReset::default(),
            protocol: ProtocolTypeSet(ProtocolType::T0, SCARD_PROTOCOL_T0),
        })
    }
    fn get_readerlist(&self) -> Result<Vec<String>, SmartcardError> {
        let mut auto_allocate = SCARD_AUTOALLOCATE;
        let reader_list = unsafe {
            let mut reader_str: *mut u16 = std::ptr::null_mut();
            // リーダーリスト、カードリストの格納形式は
            // Reader1\0Reader2\0Reader3\0...\0ReaderN\0\0 の形式で入っている。
            let ret = SCardListReadersW(
                self.ctx,
                std::ptr::null(),
                (&mut reader_str as *mut *mut u16) as *mut u16,
                &mut auto_allocate,
            );
            if ret != SCARD_S_SUCCESS {
                return Err(SmartcardError::new(
                    SmartcardErrorKind::ReaderDetectionFailed,
                ));
            }
            let reader_list = u16_ptr_to_strarray(reader_str);
            SCardFreeMemory(self.ctx, reader_str as *const winapi::ctypes::c_void);
            reader_list
        };
        Ok(reader_list)
    }
    fn get_cardlist(&self) -> Result<Vec<String>, SmartcardError> {
        let mut auto_allocate = SCARD_AUTOALLOCATE;
        let card_list = unsafe {
            let mut cards_str: *mut u16 = std::ptr::null_mut();
            // リーダーリスト、カードリストの格納形式は
            // Reader1\0Reader2\0Reader3\0...\0ReaderN\0\0 の形式で入っている。
            let ret = SCardListCardsW(
                self.ctx,
                std::ptr::null(),
                std::ptr::null(),
                0,
                (&mut cards_str as *mut *mut u16) as *mut u16,
                &mut auto_allocate,
            );
            if ret != SCARD_S_SUCCESS {
                return Err(SmartcardError::new(SmartcardErrorKind::CardNotAvailable));
            }
            let card_list = u16_ptr_to_strarray(cards_str);
            SCardFreeMemory(self.ctx, cards_str as *const winapi::ctypes::c_void);
            card_list
        };
        Ok(card_list)
    }
    // TODO : 後でいい感じに標準入力（数値）実装する
    fn show_user_prompt(reader_list: &Vec<String>) -> Option<usize> {
        let mut idx = 0;
        let reader_cnt = reader_list.len();
        if reader_cnt == 1 {
            return Some(0);
        } else if reader_cnt == 0 {
            return None;
        }
        println!("Reader# : ReaderName");
        for reader in reader_list {
            println!("{} : {}", idx, reader);
            idx += 1;
        }
        // このあたりから入力のコードを書く
        // 返却値も入力由来のものにする。
        Some(WinScardNFC::input_number(idx))
    }
    fn input_number(max: usize) -> usize {
        let mut parsed_num = 0;
        loop {
            print!("input Reader# > ");
            let _ = std::io::stdout().flush();
            let mut buf = String::new();
            if std::io::stdin().read_line(&mut buf).is_err() {
                return parsed_num;
            }
            let parsed = buf
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>()
                .parse();
            if parsed.is_ok() {
                let num = parsed.unwrap();
                if num < max {
                    parsed_num = num;
                    break;
                }
            }
        }
        parsed_num
    }

    fn set_protocol(&mut self, protocol: ProtocolType) {
        self.protocol = match protocol {
            ProtocolType::InActive => self.protocol,
            ProtocolType::T0 => ProtocolTypeSet(ProtocolType::T0, SCARD_PROTOCOL_T0),
            ProtocolType::T1 => ProtocolTypeSet(ProtocolType::T1, SCARD_PROTOCOL_T1),
            ProtocolType::T0T1 => {
                ProtocolTypeSet(ProtocolType::T0T1, SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1)
            }
            ProtocolType::RAW => ProtocolTypeSet(ProtocolType::RAW, SCARD_PROTOCOL_RAW),
            ProtocolType::Unknown => panic!("set_protocol : {:?} is Unknown protocol", protocol),
        }
    }
    fn parse_atr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // ATRの取得
        let mut reader_names = vec![0u16; 1024];
        let mut reader_name_len = reader_names.len() as u32;
        let mut scard_state = 0;
        let mut protocol = 0;
        let mut atr = [0u8; 32];
        let mut atr_len = atr.len() as u32;
        unsafe {
            let ret = SCardStatusW(
                self.h_scard,
                reader_names.as_mut_ptr(),
                &mut reader_name_len,
                &mut scard_state,
                &mut protocol,
                atr.as_mut_ptr(),
                &mut atr_len,
            );
        }
        self.atr = AnswerToReset::new(&atr)?;
        Ok(())
    }
}

fn lookup_protocol_from_winapi(protocol: u32) -> ProtocolTypeSet {
    match protocol {
        SCARD_PROTOCOL_T0 => ProtocolTypeSet(ProtocolType::T0, SCARD_PROTOCOL_T0),
        SCARD_PROTOCOL_T1 => ProtocolTypeSet(ProtocolType::T1, SCARD_PROTOCOL_T1),
        SCARD_PROTOCOL_UNDEFINED => ProtocolTypeSet(ProtocolType::RAW, SCARD_PROTOCOL_RAW),
        _ => {
            if protocol == SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1 {
                ProtocolTypeSet(ProtocolType::T0T1, SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1)
            } else {
                ProtocolTypeSet(ProtocolType::Unknown, 0)
            }
        }
    }
}

use std::ffi::{OsStr, OsString};
use std::fmt::LowerHex;
use std::io::Write;
use std::os::windows::prelude::*;

// u16のポインタ(\0終端)からOsStringへの変換
unsafe fn u16_ptr_to_string(ptr: *const u16) -> (OsString, usize) {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    (OsString::from_wide(slice), len)
}

// <string>\0<string2>\0<string3>\0\0 のようなu16のポインタが指す文字列を、Vecに変換する
fn u16_ptr_to_strarray(ptr: *const u16) -> Vec<String> {
    let mut s = Vec::new();
    let mut off = 0;
    loop {
        let (s0, len) = unsafe { u16_ptr_to_string(ptr.offset(off)) };
        if len == 0 {
            break;
        }
        off += len as isize + 1;
        s.push(s0.into_string().unwrap());
    }
    s
}

#[derive(Debug, PartialEq)]
enum TransmitErrorKind {
    Success(u8, u8),
    Warn(u8, u8),
    Error(u8, u8),
    ApiError(HRESULT),
}
#[derive(Debug)]
struct TransmitError {
    code: TransmitErrorKind,
}
impl TransmitError {
    fn new(code: TransmitErrorKind) -> Self {
        TransmitError { code }
    }
}
impl std::error::Error for TransmitError {}
impl std::fmt::Display for TransmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl smart_card::Smartcard for WinScardNFC {
    fn get_atr(&self) -> &AnswerToReset {
        &self.atr
    }
    fn version_str(&self) -> Option<String> {
        Some("0.0.0.1".to_owned())
    }
    fn version(&self) -> Option<SmartcardVersion> {
        None
    }
    fn connect_reader(
        &mut self,
        con_method: SmartcardConnectMethod,
    ) -> Result<ProtocolType, SmartcardError> {
        // リーダライタの一覧を取得する
        let reader_list = self.get_readerlist()?;
        if reader_list.len() == 0 {
            return Err(SmartcardError::new(SmartcardErrorKind::ReaderNotAvailable));
        }
        // リーダライタの一覧から選択する
        let reader_string = match con_method {
            SmartcardConnectMethod::ListIdx(idx) => {
                // idx
                let t = &reader_list[idx];
                t
            }
            SmartcardConnectMethod::UserPrompt => {
                let mut id_string = "";
                loop {
                    match WinScardNFC::show_user_prompt(&reader_list) {
                        Some(idx) => {
                            id_string = &reader_list[idx];
                        }
                        None => {
                            // リーダーがない
                            break;
                        }
                    };
                    break;
                }
                id_string
            }
        };

        unsafe {
            let wait_target: Vec<u16> = OsStr::new(reader_string)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect();
            let mut read_state = vec![SCARD_READERSTATEW {
                szReader: wait_target.as_ptr(),
                pvUserData: std::ptr::null_mut(),
                dwCurrentState: SCARD_STATE_UNAWARE,
                dwEventState: 0,
                cbAtr: 0,
                rgbAtr: [0u8; 36],
            }];
            SCardGetStatusChangeW(self.ctx, 0, read_state.as_mut_ptr(), 1);
            if read_state[0].cbAtr == 0 {
                println!("カードリーダーにカードを置いてください。");
                read_state[0].dwCurrentState = read_state[0].dwEventState;
                // 正しく読める状態になるまでリーダーを待機する
                SCardGetStatusChangeW(self.ctx, INFINITE, read_state.as_mut_ptr(), 1);
            }
            let reader_string: Vec<u16> = OsStr::new(reader_string)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect();
            let mut active_protocol = 0;
            let mut state = Err(SmartcardError::new(SmartcardErrorKind::NotReady));
            // Auto-negotiation
            for protocol in &[ProtocolType::T0T1, ProtocolType::T0, ProtocolType::T1] {
                self.config_protocol(*protocol);
                let ret = SCardConnectW(
                    self.ctx,
                    reader_string.as_ptr(),
                    SCARD_SHARE_SHARED, // 共有モード（SCardBeginTransaction(hCard)～SCardEndTransaction(hCard, SCARD_LEAVE_CARD);でトランザクションを実施する）
                    self.protocol.1,
                    &mut self.h_scard,
                    &mut active_protocol,
                );
                match ret {
                    SCARD_S_SUCCESS => {
                        self.protocol = lookup_protocol_from_winapi(active_protocol);
                        state = Ok(self.protocol.0);
                        break;
                    }
                    SCARD_E_PROTO_MISMATCH => {
                        continue;
                    }
                    _ => {
                        state = Err(SmartcardError::new(SmartcardErrorKind::ReaderNotAvailable));
                        break;
                    }
                };
            }
            self.parse_atr();
            state
        }
    }
    /// コマンドの送信
    fn transmit(&self, data: Box<dyn APDU>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = data.read8();
        let mut res = vec![0u8; 256];
        let mut size = res.len() as u32;

        unsafe {
            let state = SCardTransmit(
                self.h_scard,
                &self.protocol.get_protocol(),
                data.as_ptr(),
                data.len() as u32,
                std::ptr::null_mut(),
                res.as_mut_ptr(),
                &mut size,
            );
            if state != SCARD_S_SUCCESS {
                return Err(Box::new(TransmitError::new(TransmitErrorKind::ApiError(
                    state,
                ))));
            }
        }
        let code_start = size as usize - 2;
        let code_end = size as usize;
        let code = &res[code_start..code_end];
        if code == &[0x90, 0x00] {
            Ok(res[0..code_start].to_vec())
        } else {
            match code[0] {
                0x62 => Err(Box::new(TransmitError::new(TransmitErrorKind::Error(
                    code[0], code[1],
                )))),
                _ => Err(Box::new(TransmitError::new(TransmitErrorKind::Warn(
                    code[0], code[1],
                )))),
            }
        }
        // SCardGetStatusChange 的なやつを呼び出してReadyになるまで待機させたりすればいいけど
        // とりあえず今はなんだかよくわからんがとにかくコマンド叩く　よし！
        // 実際にStateChangeを呼ぶ場合には、カードリーダーの数分のステート管理領域を生成する必要がある。
        // 一旦分離とか難しいことを考えずに、そのままAPI呼び出す感じで実装してみる。
        // リファクタはあとからやろう。
    }
    /// プロトコルを設定すると現在アクティブなプロトコルが返却される。
    /// 現在アクティブなプロトコルを知りたい場合や明示的に変更をしない場合は
    /// ProtocolType::InActive を使うと良い。
    fn config_protocol(&mut self, protocol: ProtocolType) -> Option<ProtocolType> {
        self.set_protocol(protocol);
        Some(self.protocol.0)
    }
}

impl Drop for WinScardNFC {
    fn drop(&mut self) {
        // drop
        unsafe {
            SCardDisconnect(self.h_scard, SCARD_LEAVE_CARD);
            SCardReleaseContext(self.ctx);
        }
    }
}
