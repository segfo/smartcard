// NFCのFactoryメソッド
// SCardAPIとLibNFCに対応するようにする。
use nfc::*;
pub enum FactoryType{
    WindowsScardAPI,LibMFC
}

pub struct NfcFactory{}
impl NfcFactory{
    fn create_nfc_instance(ftype:FactoryType)->NFC{
        match(ftype){
            FactoryType::WindowsScardAPI => {
                
            },
            FactoryType::LibMFC => {

            } 
        }
    }
}