// Windows SCardAPIによる実装を記述する
use crate::smart_card;
use crate::smart_card::*;
use winapi::um::{winscard::*,winsmcrd::*};
use winapi::shared::winerror::*;

#[derive(Clone,Copy)]
struct ProtocolTypeSet(ProtocolType,u32);

pub struct WinScardNFC{
    ctx : SCARDCONTEXT,
    h_scard : SCARDHANDLE,
    protocol : ProtocolTypeSet
}

struct Reader{
    atr_string:String
}

impl WinScardNFC{
    pub fn new()->Result<Self,SmartcardError>{
        let ctx = unsafe{
            let mut ctx:SCARDCONTEXT=0;
            let ret = SCardEstablishContext(SCARD_SCOPE_USER, std::ptr::null(), std::ptr::null(),&mut ctx);
            if ret != SCARD_S_SUCCESS{
                return Err(SmartcardError::new(SmartcardErrorKind::ResMgrCtxInit))
            }
            ctx
        };
        Ok(WinScardNFC{
            ctx:ctx,
            h_scard:0,
            protocol:ProtocolTypeSet(
                ProtocolType::T0T1,SCARD_PROTOCOL_T0|SCARD_PROTOCOL_T1
            )
        })
    }
    fn get_readerlist(&self)->Result<Vec<String>,SmartcardError>{
        let mut auto_allocate = SCARD_AUTOALLOCATE;
        let reader_list = unsafe{
            let mut reader_str:*mut u16 = std::ptr::null_mut();
            // リーダーリスト、カードリストの格納形式は
            // Reader1\0Reader2\0Reader3\0...\0ReaderN\0\0 の形式で入っている。
            let ret = SCardListReadersW(self.ctx, std::ptr::null(), (&mut reader_str as *mut *mut u16) as *mut u16 , &mut auto_allocate);
            if ret != SCARD_S_SUCCESS{
                return Err(SmartcardError::new(SmartcardErrorKind::ReaderDetectionFailed))
            }
            let reader_list = u16_ptr_to_strarray(reader_str);
            SCardFreeMemory(self.ctx, reader_str as *const winapi::ctypes::c_void);
            reader_list
        };
        Ok(reader_list)
    }
    fn get_cardlist(&self)->Result<Vec<String>,SmartcardError>{
        let mut auto_allocate = SCARD_AUTOALLOCATE;
        let card_list = unsafe{
            let mut cards_str:*mut u16 = std::ptr::null_mut();
            // リーダーリスト、カードリストの格納形式は
            // Reader1\0Reader2\0Reader3\0...\0ReaderN\0\0 の形式で入っている。
            let ret = SCardListCardsW(self.ctx, std::ptr::null(), std::ptr::null(), 0, (&mut cards_str as *mut *mut u16) as *mut u16, &mut auto_allocate);
            if ret != SCARD_S_SUCCESS{
                return Err(SmartcardError::new(SmartcardErrorKind::CardNotAvailable))
            }
            let card_list = u16_ptr_to_strarray(cards_str);
            SCardFreeMemory(self.ctx, cards_str as *const winapi::ctypes::c_void);
            card_list
        };
        Ok(card_list)
    }
    // TODO : 後でいい感じに標準入力（数値）実装する
    fn show_user_prompt(reader_list:&Vec<String>)->Option<usize>{
        let mut idx = 1;
        println!("Reader# : ReaderName");
        for reader in reader_list{
            println!("{} : {}",idx,reader);
            idx += 1;
        }
        print!("input Reader# > ");
        // このあたりから入力のコードを書く
        // 返却値も入力由来のものにする。
        Some(0)
    }
    fn set_protocol(&mut self,protocol:ProtocolType){
        self.protocol = match protocol{
            ProtocolType::InActive => self.protocol,
            ProtocolType::T0 => ProtocolTypeSet(ProtocolType::T0,SCARD_PROTOCOL_T0),
            ProtocolType::T1 => ProtocolTypeSet(ProtocolType::T1,SCARD_PROTOCOL_T1),
            ProtocolType::T0T1 => ProtocolTypeSet(ProtocolType::T0T1,SCARD_PROTOCOL_T0|SCARD_PROTOCOL_T1),
            ProtocolType::RAW => ProtocolTypeSet(ProtocolType::RAW,SCARD_PROTOCOL_RAW),
            ProtocolType::Unknown => panic!("set_protocol : {:?} is Unknown protocol",protocol)
        }
    }
}

fn lookup_protocol_from_winapi(protocol:u32)->ProtocolTypeSet{
    match protocol{
        SCARD_PROTOCOL_T0 => ProtocolTypeSet(ProtocolType::T0,SCARD_PROTOCOL_T0),
        SCARD_PROTOCOL_T1 => ProtocolTypeSet(ProtocolType::T1,SCARD_PROTOCOL_T1),
        SCARD_PROTOCOL_UNDEFINED => ProtocolTypeSet(ProtocolType::RAW,SCARD_PROTOCOL_RAW),
        _ => {
            if protocol == SCARD_PROTOCOL_T0|SCARD_PROTOCOL_T1{
                ProtocolTypeSet(ProtocolType::T0T1,SCARD_PROTOCOL_T0|SCARD_PROTOCOL_T1)
            }else{
                ProtocolTypeSet(ProtocolType::Unknown,0)
            }
        }
    }
}

use std::ffi::{OsString,OsStr};
use std::os::windows::prelude::*;

// u16のポインタ(\0終端)からOsStringへの変換
unsafe fn u16_ptr_to_string(ptr: *const u16) -> (OsString,usize) {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    (OsString::from_wide(slice),len)
}

// <string>\0<string2>\0<string3>\0\0 のようなu16のポインタが指す文字列を、Vecに変換する
fn u16_ptr_to_strarray(ptr:*const u16)->Vec<String>{
    let mut s = Vec::new();
    let mut off = 0;
    loop{
        let (s0,len) = unsafe { u16_ptr_to_string(ptr.offset(off)) };
        if len == 0 { break; }
        off += len as isize + 1;
        s.push(s0.into_string().unwrap());
    }
    s
}

impl smart_card::Smartcard for WinScardNFC{
    fn version_str(&self)->Option<&str>{
        None
    }
    fn version(&self)->Option<SmartcardVersion>{
        None
    }
    fn connect_reader(&mut self,con_method:SmartcardConnectMethod)->Result<ProtocolType, SmartcardError>{
        // リーダライタの一覧を取得する
        let reader_list = self.get_readerlist().unwrap();
        // リーダライタの一覧から選択する
        let reader_string = match con_method{
            SmartcardConnectMethod::ListIdx(idx)=>{
                // idx
                 let t = &reader_list[idx];
                 t
            },
            SmartcardConnectMethod::UserPrompt=>{
                let mut id_string = "";
                loop{
                    match WinScardNFC::show_user_prompt(&reader_list){
                        Some(idx)=>{
                            id_string = &reader_list[idx];
                        },
                        None=>{continue;}
                    };
                    break;
                }
                id_string
            }
        };

        unsafe {
            let reader_string:Vec<u16>= OsStr::new(reader_string).encode_wide().chain(Some(0).into_iter()).collect();
            let mut active_protocol = 0;
            // Auto-negotiation
            let mut state = Ok(ProtocolType::InActive);
            for protocol in &[ProtocolType::T0,ProtocolType::T1,ProtocolType::T0T1]{
                self.config_protocol(*protocol);
                let ret = SCardConnectW(self.ctx, reader_string.as_ptr(), SCARD_SHARE_SHARED, self.protocol.1, &mut self.h_scard, &mut active_protocol);
                state = match ret{
                    SCARD_S_SUCCESS=>{
                        self.protocol = lookup_protocol_from_winapi(active_protocol);
                        state = Ok(self.protocol.0);
                        break;
                    },
                    SCARD_E_NOT_READY=>{Err(SmartcardError::new(SmartcardErrorKind::NotReady))},
                    SCARD_E_PROTO_MISMATCH=>{
                        continue;
                    },
                    _=>{
                        println!("API Return : {:?}",ret);
                        state = Err(SmartcardError::new(SmartcardErrorKind::ReaderNotAvailable));
                        break;
                    }
                };
            }
            state
        }
    }
    /// コマンドの送信
    fn transmit(&self,data:Box<dyn APDU>){
        
    }
    /// プロトコルを設定すると現在アクティブなプロトコルが返却される。
    /// 現在アクティブなプロトコルを知りたい場合や明示的に変更をしない場合は
    /// ProtocolType::InActive を使うと良い。
    fn config_protocol(&mut self,protocol:ProtocolType)->Option<ProtocolType>{
        self.set_protocol(protocol);
        Some(self.protocol.0)
    }
}

impl Drop for WinScardNFC{
    fn drop(&mut self){
        // drop
        unsafe{
            SCardDisconnect(self.h_scard,SCARD_LEAVE_CARD);
            SCardReleaseContext(self.ctx);
        }
    }
}