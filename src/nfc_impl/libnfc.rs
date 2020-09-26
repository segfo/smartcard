// LibNFCによる実装を記述する
use nfc::context;
use nfc::misc;
use nfc::device;
use crate::smart_card::*;

pub struct NFClibnfc{
    ctx:*mut nfc::ffi::nfc_context
}
impl NFClibnfc{
    pub fn new()->Result<Self,Box<dyn std::error::Error>>{
        let mut context = context::new();
        if context.is_null() {
            return Err(
                Box::new(SmartcardError::new(SmartcardErrorKind::ContextInit))
            );
        }
        Ok(NFClibnfc{
            ctx:context
        })
    }
    fn baselib_version_str(&self)->&str{
        misc::version()
    }
}

impl    Smartcard for NFClibnfc{
    fn version_str(&self)->Option<&str>{
        Some(self.baselib_version_str())
    }
    fn version(&self)->Option<SmartcardVersion>{
        None
    }
}

