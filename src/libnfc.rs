// LibNFCによる実装を記述する
use nfc::context;
use nfc::misc;
use nfc::device;
use crate::nfc::*;

pub struct NFClibnfc{
    ctx:*mut nfc::ffi::nfc_context
}
impl NFClibnfc{
    pub fn new()->Result<Self,Box<dyn std::error::Error>>{
        let mut context = context::new();
        if context.is_null() {
            return Err(
                Box::new(NFCError::new(NFCErrorKind::ContextInit))
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

impl    NFC for NFClibnfc{
    fn version_str(&self)->Option<&str>{
        Some(self.baselib_version_str())
    }
    fn version(&self)->Option<NFCVersion>{
        None
    }
}

