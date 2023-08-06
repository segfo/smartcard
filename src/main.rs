mod apdu_contactless;
mod nfc_impl;
mod pc_sc_standard;
mod smart_card;
use std::fmt::LowerHex;

use nfc_impl::NfcFactory;
use pc_sc_standard::CardType;
use smart_card::Smartcard;

use crate::{
    pc_sc_standard::{ApduBuilderExtWithFelica, ApduBuilderExtWithPcsc3V2},
    smart_card::ProtocolType,
};
//mod nfc_nullimpl;
//use crate::smart_card::*;
//mod libnfc;
//use crate::libnfc::*;

fn main() {
    let mut nfc: Box<dyn smart_card::Smartcard> =
        NfcFactory::create_nfc_instance(nfc_impl::FactoryType::WindowsScardAPI);
    nfc.connect_reader(smart_card::SmartcardConnectMethod::UserPrompt)
        .unwrap();
    let apdu = apdu_contactless::ApduBuilder::new().get_card_kind().build();
    let card_kind = match nfc.transmit(Box::new(apdu)) {
        Ok(card_kind) => CardType::from(card_kind[0]),
        Err(_) => CardType::UnknownCard,
    };
    if card_kind == CardType::FeliCa {
        read_felica();
    } else {
        println!("このカードはFeliCaではありません。");
    }
    // nfc.control();
    show_card_info(&mut nfc, card_kind);
}

fn read_felica() {}

fn show_card_info(nfc: &mut Box<dyn Smartcard>, card_type: CardType) {
    let card_kind_num = card_type as u8;
    let card_kind = [
        "不明なカード",
        "ISO14443A (MIFARE Classic / Ultralight)",
        "ISO14443B",
        "PicoPassB",
        "FeliCa",
        "NFC Type 1 Tag",
        "Mifare Emulation Card",
        "ISO14443-4A (MIFARE DESFire EV1 / MIFARE DESFire v0.6 / SmartMX)",
        "ISO14443-4B (Infineon / Atmel /ST microelectronics)",
        "TypeA NFC-DEP ターゲット",
        "FeliCa NFC-DEP ターゲット",
    ];
    println!("ライブラリのバージョン: {}", nfc.version_str().unwrap());
    let atr = nfc.get_atr();
    println!("[Answer to response(ATR)]");
    println!("    カード名: {}", atr.card_name.as_ref().unwrap().0);
    println!(
        "    管理情報バイト: {}",
        hex_dump(atr.historical_data.as_ref().unwrap())
    );
    let apdu = apdu_contactless::ApduBuilder::new().get_serial().build();
    let mode_str = match nfc.config_protocol(ProtocolType::InActive) {
        Some(ProtocolType::T1) => "T1(ブロック転送モード)",
        Some(ProtocolType::T0) => "T0(キャラクタ転送モード)",
        Some(ProtocolType::RAW) => "RAWモード",
        _ => "<不明なプロトコルです>",
    };
    println!("確立したプロトコル: {}", mode_str);
    let i = match card_kind_num {
        1 => 1,
        4 => 2,
        8 => 3,
        _ => 0,
    };

    if let Ok(result) = nfc.transmit(Box::new(apdu)) {
        println!(
            "{}: {}",
            ["固有ID", "UID", "IDm", "PUPI"][i],
            hex_dump(&result)
        );
    }
    let apdu = apdu_contactless::ApduBuilder::new().get_ats().build();
    match nfc.transmit(Box::new(apdu)) {
        Ok(ats) => {
            if ats.len() > 0 {
                println!(
                    "{}: {}",
                    ["不明なID", "ATS", "PMm", "ATS"][i],
                    hex_dump(&ats)
                );
                if i == 2 {
                    println!("ROM種別: {:02x}", ats[0]);
                    println!("IC種別: {:02x}", ats[1]);
                }
            } else {
                println!("ATS: 情報無し(コマンドは成功しました)")
            }
        }
        Err(_) => {
            println!("ATS: 読み出し非対応")
        }
    }
    if card_kind_num as usize <= card_kind.len() {
        println!("カード種別: {}", card_kind[card_kind_num as usize]);
    } else {
        println!("カード種別: 不明({:02x})", card_kind_num);
    }
    let apdu = apdu_contactless::ApduBuilder::new().get_card_name().build();
    if let Ok(card_name) = nfc.transmit(Box::new(apdu)) {
        if card_name.len() > 0 {
            println!("カード名: {}", String::from_utf8(card_name).unwrap());
        }
    }
    let apdu = apdu_contactless::ApduBuilder::new()
        .get_card_kind_name()
        .build();
    if let Ok(kind_name) = nfc.transmit(Box::new(apdu)) {
        if kind_name.len() > 0 {
            println!("種別名: {}", String::from_utf8(kind_name).unwrap());
        }else{
            println!("種別名: 不明");
        }
    }else{
        println!("種別名: 不明");
    }
    let apdu = apdu_contactless::ApduBuilder::new().get_card_id().build();
    if let Ok(card_id) = nfc.transmit(Box::new(apdu)) {
        if card_id.len() > 0 {
            println!("カードID: {}", hex_dump(&card_id));
        }else{
            println!("カードID: 不明");
        }
    }else{
        println!("カードID: 不明");
    }
}

fn hex_dump<T: LowerHex>(data: &Vec<T>) -> String {
    let mut s = data.iter().map(|d| format!("{d:02x}-")).collect::<String>();
    s.pop();
    s
}
