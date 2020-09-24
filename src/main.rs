mod nfc;
mod nfc_nullimpl;
use crate::nfc::*;
mod libnfc;
use crate::libnfc::*;

fn main() {
    let nfc = NFClibnfc::new().unwrap();
    // Print libnfc version
    println!("libnfc version: {}", nfc.version_str().unwrap());
}
