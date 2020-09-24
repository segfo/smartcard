// NFC Trait
pub trait NfcDto{}
pub trait TransmitDTO{}
pub enum NFCConnectMethod{
    Default,ListIdx(usize),ID(String)
}
pub trait NFC{
    fn version_str(&self)->Option<&str>;
    fn version(&self)->Option<NFCVersion>;
    fn connect_reader(&self);
    fn transmit(dto:TransmitDTO)
//    fn set_protocol_version(&self,dto:dyn NfcDto);
}
#[derive(Debug,Clone,Copy)]
pub struct NFCVersion{
    pub major:u32,
    pub minor:u32,
    pub build:u32,
    pub revision:u32,
}

impl NFCVersion{
    pub fn new(major:u32,minor:u32,build:u32,revision:u32)->Self{
        NFCVersion{
            major:major,
            minor:minor,
            build:build,
            revision:revision
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub enum NFCErrorKind{
    Success,ContextInit,
}

#[derive(Debug)]
pub struct NFCError{
    msg:String,
    kind:NFCErrorKind
}

impl NFCError{
    pub fn new(kind:NFCErrorKind)->Self{
        NFCError{
            msg:NFCError::kind2msg(kind),
            kind:kind
        }
    }
    fn kind2msg(kind:NFCErrorKind)->String{
        let msg = match kind{
            NFCErrorKind::Success=>"",
            NFCErrorKind::ContextInit=>"Unable to initialize new NFC context!"
        };
        msg.to_owned()
    }
}

impl std::fmt::Display for NFCError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",self.msg)
    }
}

impl std::error::Error for NFCError{}
