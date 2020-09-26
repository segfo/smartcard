use crate::smart_card::*;
pub struct NFCNull{

}
impl NFCNull{
    pub fn new()->Self{
        NFCNull{}
    }
}

impl Smartcard for NFCNull{
    fn version_str(&self)->Option<&str>{
        return Some("NFC Null 0.0.0.1")
    }
    fn version(&self)->Option<SmartcardVersion>{
        return Some(
            SmartcardVersion::new(0,0,0,2)
        )
    }
}