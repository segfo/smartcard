mod smart_card;
mod nfc_impl;
use nfc_impl::NfcFactory;
//mod nfc_nullimpl;
//use crate::smart_card::*;
//mod libnfc;
//use crate::libnfc::*;

fn main() {
    let nfc:Box<smart_card::Smartcard> = NfcFactory::create_nfc_instance(nfc_impl::FactoryType::WindowsScardAPI);
    
    println!("version: {}", nfc.version_str().unwrap());
}
