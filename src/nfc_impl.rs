// NFCのFactoryメソッド
// SCardAPIとLibNFCに対応するようにする。
use crate::smart_card::*;
mod nfc_winscard;
pub enum FactoryType{
    WindowsScardAPI,LibMFC
}

pub struct NfcFactory{}
impl NfcFactory{
    pub fn create_nfc_instance(ftype:FactoryType)->Box<dyn Smartcard>{
        match ftype{
            FactoryType::WindowsScardAPI => {
                Box::new(crate::nfc_impl::nfc_winscard::WinScardNFC::new().unwrap())
            },
            FactoryType::LibMFC => {
                unimplemented!()
            } 
        }
    }
}