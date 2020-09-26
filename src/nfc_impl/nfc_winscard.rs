// Windows SCardAPIによる実装を記述する
use crate::smart_card;
use crate::smart_card::*;
use winapi::um::winscard::*;
use winapi::shared::winerror::*;

pub struct WinScardNFC{
    ctx:SCARDCONTEXT
}

struct Reader{
    atr_string:String
}

impl WinScardNFC{
    pub fn new()->Result<Self,SmartcardError>{
        let ctx = unsafe{
            let mut ctx:SCARDCONTEXT=0;
            let ret = SCardEstablishContext(SCARD_SCOPE_USER, std::ptr::null(), std::ptr::null(),&mut ctx);
            println!("{}:{} : ret : {:x}  ctx : {:x}",file!(),line!(),ret,ctx);
            if ret != SCARD_S_SUCCESS{
                return Err(SmartcardError::new(SmartcardErrorKind::ResMgrCtxInit))
            }
            ctx
        };
        Ok(WinScardNFC{ctx:ctx})
    }
    fn get_readerlist(&self)->Vec<Reader>{
        let reader_list = Vec::new();
        let mut cards = SCARD_AUTOALLOCATE;
        unsafe{
            let mut cards_str: u16=0;
            SCardListCardsW(self.ctx, std::ptr::null(), std::ptr::null(), 0, &mut cards_str, &mut cards);
        }
        reader_list
    }
}

impl smart_card::Smartcard for WinScardNFC{
    fn version_str(&self)->Option<&str>{
        None
    }
    fn version(&self)->Option<SmartcardVersion>{
        None
    }
    fn connect_reader(&self,con_method:SmartcardConnectMethod)->Result<(), SmartcardError>{
        // リーダライタの一覧を取得する
        let list = self.get_readerlist();
        // リーダライタの一覧から選択する
        match con_method{
            SmartcardConnectMethod::ID(id_str)=>{
                // id_str;
            },
            SmartcardConnectMethod::ListIdx(idx)=>{
                // idx
            },
            SmartcardConnectMethod::Default=>{
                // 一番最初のReaderを使う
            }
        };                                                                              
        //SCardConnectW(self.ctx, szReader: LPCWSTR, dwShareMode: DWORD, dwPreferredProtocols: DWORD, phCard: LPSCARDHANDLE, pdwActiveProtocol: LPDWORD)
        unimplemented!()
    }
    /// コマンドの送信
    fn transmit(&self,data:Box<dyn APDU>){

    }
    /// プロトコルを設定すると現在アクティブなプロトコルが返却される。
    /// 現在アクティブなプロトコルを知りたい場合や明示的に変更をしない場合は
    /// ProtocolType::InActive を使うと良い。
    fn config_protocol(&self,protocol:ProtocolType)->Option<ProtocolType>{
        let p = match protocol{
            ProtocolType::InActive => ProtocolType::T0,
            ProtocolType::T0 => ProtocolType::T0,
            ProtocolType::T1 => ProtocolType::T1,
            ProtocolType::RAW => ProtocolType::RAW
        };
        Some(p)
    }
}

impl Drop for WinScardNFC{
    fn drop(&mut self){
        // drop
        unsafe{SCardReleaseContext(self.ctx);}
    }
}