use std::cmp::PartialEq;

use crate::smart_card::APDU;
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Classes {
    IsoFullAccording = 0x00, // コマンドレスポンス及び符号化方式のISO準拠
    StructAndFlagsIsoAccordingProprietarySemanticsAndEncoding1 = 0x80, // コマンドとレスポンスの構造及び下位8ビットのフラグがISOに準拠している。リクレスの符号化方式及び意味は独自に実装して良い。
    StructAndFlagsIsoAccordingProprietarySemanticsAndEncoding2 = 0x90, // コマンドとレスポンスの構造及び下位8ビットのフラグがISOに準拠している。リクレスの符号化方式及び意味は独自に実装して良い。
    IsoPrincipleFullAccording = 0xA0,                                  // 原則としてISO準拠とする。
    StructIsoAccording1 = 0xB0, // コマンドレスポンス構造のみISOに準拠している。下位8ビット及び、リクレス符号化及び意味は独自に実装して良い。
    StructIsoAccording2 = 0xC0, // コマンドレスポンス構造のみISOに準拠している。下位8ビット及び、リクレス符号化及び意味は独自に実装して良い。
    Proprietary1 = 0xD0,        // 完全にISOに準拠しない独自符号化
    Proprietary2 = 0xE0,        // 完全にISOに準拠しない独自符号化
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SecureMessaging {
    Plain = 0,             // 何もしない
    Proprietary = 1,       // 個別利用(独自拡張)セキュアメッセージ
    Clause6 = 2,           // 箇条 6 ヘッダ認証なし
    Clause6HeaderAuth = 3, // 箇条 6 ヘッダ認証あり
    Undefined,             // 未定義
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instructions {
    EraseBinary = 0x0E,
    Verify = 0x20,
    ManageChannel = 0x70,
    ExternalAuthenticate = 0x82,
    GetChallenge = 0x84,
    InternalAuthenticate = 0x88,
    SelectFile = 0xA4,
    ReadBinary = 0xB0,
    ReadRecords = 0xB2,
    GetResponse = 0xC0,
    Envelope = 0xC2,
    GetData = 0xCA,
    WriteBinary = 0xD0,
    WriteRecord = 0xD2,
    UpdateBinary = 0xD6,
    PutData = 0xDA,
    UpdateData = 0xDC,
    AppendRecord = 0xE2,
}

/**
 * Apdu
 * APDUビルダ構造体
 */
#[derive(Debug, Clone)]
pub struct ApduBuilder {
    cla: u8,
    ins: u8,
    parameter: [u8; 2],
    data_field: Option<Vec<u8>>,
}
impl ApduBuilder {
    pub fn new() -> Self {
        ApduBuilder {
            cla: 0,
            ins: 0,
            parameter: [0, 0],
            data_field: None,
        }
    }
    pub fn set_ext(&mut self, use_ext_spec: bool) -> &mut Self {
        self.cla |= if use_ext_spec { 0x80 } else { 0x00 };
        self
    }
    pub fn set_raw_classs_code(&mut self, class_code: u8) -> &mut Self {
        self.cla = class_code;
        self
    }
    pub fn set_vchannel(&mut self, vchanel_no: u8) -> &mut Self {
        if 0 <= vchanel_no && vchanel_no <= 3 {
            self.cla &= 0xff ^ 0b0000_0011;
            self.cla |= vchanel_no;
        }
        self
    }
    pub fn get_vchannel(&self) -> u8 {
        self.cla & 0x03
    }
    pub fn is_command_chain(&mut self) -> bool {
        // iso7816では、コマンドチェインは実装されていない。
        false
    }
    pub fn set_secure_mode(&mut self, secure_type: SecureMessaging) -> &mut Self {
        self.cla &= 0xff ^ 0b0000_1100;
        self.cla |= (secure_type as u8) << 2;
        self
    }
    pub fn get_secure_mode(&self) -> SecureMessaging {
        match ((self.cla & 0b0000_1100) >> 2) & 0b11 {
            0 => SecureMessaging::Plain,
            1 => SecureMessaging::Proprietary,
            2 => SecureMessaging::Clause6,
            3 => SecureMessaging::Clause6HeaderAuth,
            _ => SecureMessaging::Undefined,
        }
    }
    pub fn build(&self) -> Apdu {
        Apdu {
            cla: self.cla,
            ins: self.ins,
            parameter: self.parameter,
            data_field: self.data_field.clone(),
        }
    }
}
use crate::pc_sc_standard::*;
impl ApduBuilderExtWithPcsc3V2 for ApduBuilder {
    fn get_ats(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[1, 0];
        self.data_field=Some(vec![0]);
        self
    }
    fn get_serial(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[0, 0];
        self.data_field=Some(vec![0]);
        self
    }
}

impl ApduBuilderExtWithFelica for ApduBuilder{
    fn get_card_id(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[0xF0, 0];
        self.data_field=Some(vec![0]);
        self
    }
    fn get_card_name(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[0xF1, 0];
        self.data_field=Some(vec![0]);
        self
    }
    fn get_card_kind(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[0xF3, 0];
        self.data_field=Some(vec![0]);
        self
    }
    fn get_card_kind_name(&mut self) -> &mut Self {
        self.cla=0xFF;
        self.ins=0xCA;
        self.parameter=[0xF4, 0];
        self.data_field=Some(vec![0]);
        self
    }
}

#[derive(Debug)]
pub struct Apdu {
    cla: u8,
    ins: u8,
    parameter: [u8; 2],
    data_field: Option<Vec<u8>>,
}
impl APDU for Apdu {
    fn read8(&self) -> Vec<u8> {
        if self.data_field.is_none() {
            [self.cla, self.ins]
                .iter()
                .cloned()
                .chain(self.parameter.iter().cloned())
                .collect::<Vec<u8>>()
        }else{
            [self.cla, self.ins]
            .iter()
            .cloned()
            .chain(self.parameter.iter().cloned())
            .chain(self.data_field.as_ref().unwrap().iter().cloned())
            .collect::<Vec<u8>>()
        }
    }
}

// #[derive(Debug)]
// enum ApduBuildErrorKind {
//     Success,
//     InvalidInstruction,
// }
// #[derive(Debug)]
// struct ApduBuildError {
//     error_id: ApduBuildErrorKind,
// }
// impl ApduBuildError {
//     fn new(kind: ApduBuildErrorKind) -> Self {
//         ApduBuildError { error_id: kind }
//     }
// }
// impl std::error::Error for ApduBuildError {}
// impl std::fmt::Display for ApduBuildError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "ApduBuildError : {:?}", self.error_id)
//     }
// }

#[test]
fn APDU_secure_disable_1() {
    let apdu = ApduBuilder::new();
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Plain);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0);
}
#[test]
fn APDU_secure_disable_2() {
    let mut apdu = ApduBuilder::new();
    apdu.set_secure_mode(SecureMessaging::Plain);
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Plain);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0);
}

#[test]
fn APDU_secure_proprietary() {
    let mut apdu = ApduBuilder::new();
    apdu.set_secure_mode(SecureMessaging::Proprietary);
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Proprietary);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0b0000_0100);
}

#[test]
fn APDU_secure_clause6() {
    let mut apdu = ApduBuilder::new();
    apdu.set_secure_mode(SecureMessaging::Clause6);
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Clause6);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0b0000_1000);
}

#[test]
fn APDU_secure_clause6_auth() {
    let mut apdu = ApduBuilder::new();
    apdu.set_secure_mode(SecureMessaging::Clause6HeaderAuth);
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Clause6HeaderAuth);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0b0000_1100);
}
#[test]
fn APDU_secure_cause6_to_clause6_auth() {
    let mut apdu = ApduBuilder::new();
    apdu.set_secure_mode(SecureMessaging::Clause6);
    apdu.set_secure_mode(SecureMessaging::Clause6HeaderAuth);
    let mode = apdu.get_secure_mode();
    assert_eq!(mode, SecureMessaging::Clause6HeaderAuth);
    let payload = apdu.build();
    assert_eq!(payload.read8()[0], 0b0000_1100);
}
