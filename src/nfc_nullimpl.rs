use crate::nfc::*;
pub struct NFCNull{

}
impl NFCNull{
    pub fn new()->Self{
        NFCNull{}
    }
}

impl NFC for NFCNull{
    fn version_str(&self)->Option<&str>{
        return Some("NFC Null 0.0.0.1")
    }
    fn version(&self)->Option<NFCVersion>{
        return Some(
            NFCVersion::new(0,0,0,2)
        )
    }
}