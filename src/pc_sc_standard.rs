// PC/SC規格に準拠したカードとの通信に関わる定義を記述していく

use crate::apdu_iso7816;

#[derive(Debug, Clone)]
pub struct AnswerToReset {
    pub raw_atr: Option<Vec<u8>>,
    pub ss: u8,
    pub rid: Option<Vec<u8>>, // Registered application provider identifier
    pub card_name: Option<(String, CardName)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardName {
    MifareClassic1k,
    MifareClassic4k,
    MifareUltralight,
    Srix512,
    MifareMini,
    MifarePlusSl12k,
    MifarePlusSl14k,
    MifarePlusSl22k,
    MifarePlusSl24k,
    MifareUltralightC,
    TopazJewel,
    Felica,
    Jcop30,
    UnknownTagName,
}
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    UnknownCard, // 不明なカード
    Iso14443A,   // NFC-TypeA
    Iso14443B,   // NFC-TypeB
    PicoPass,
    FeliCa,
    NfcType1Tag,
    MifareEmulationCard,
    Iso14443_4A, // NFC-TypeA
    Iso14443_4B, // NFC-TypeB
    TypeANfcDepTarget,
    FeliCaNfcDepTarget,
}
const CARD_LIST: [CardType; 11] = [
    CardType::UnknownCard, // 不明なカード
    CardType::Iso14443A,   // NFC-TypeA
    CardType::Iso14443B,   // NFC-TypeB
    CardType::PicoPass,
    CardType::FeliCa,
    CardType::NfcType1Tag,
    CardType::MifareEmulationCard,
    CardType::Iso14443_4A, // NFC-TypeA
    CardType::Iso14443_4B, // NFC-TypeB
    CardType::TypeANfcDepTarget,
    CardType::FeliCaNfcDepTarget,
];

impl From<u8> for CardType {
    fn from(value: u8) -> Self {
        if usize::from(value) < CARD_LIST.len() {
            CARD_LIST[value as usize].clone()
        } else {
            CardType::UnknownCard
        }
    }
}
impl From<u16> for CardType {
    fn from(value: u16) -> Self {
        if value < 255 {
            Self::from(value as u8)
        } else {
            Self::from(0 as u8)
        }
    }
}
impl From<u32> for CardType {
    fn from(value: u32) -> Self {
        if value < 255 {
            Self::from(value as u8)
        } else {
            Self::from(0 as u8)
        }
    }
}
impl From<u64> for CardType {
    fn from(value: u64) -> Self {
        if value < 255 {
            Self::from(value as u8)
        } else {
            Self::from(0 as u8)
        }
    }
}
impl From<u128> for CardType {
    fn from(value: u128) -> Self {
        if value < 255 {
            Self::from(value as u8)
        } else {
            Self::from(0 as u8)
        }
    }
}
impl From<usize> for CardType {
    fn from(value: usize) -> Self {
        if value < 255 {
            Self::from(value as u8)
        } else {
            Self::from(0 as u8)
        }
    }
}

impl AnswerToReset {
    pub fn new(atr: &[u8; 32]) -> Result<Self, Box<dyn std::error::Error>> {
        if atr[0] != 0x3B {
            return Err(Box::new(ATRParseError::new(
                ATRParseErrorCode::InvalidHeader(atr[0]),
            )));
        }
        let (rid, ss, card_name) = AnswerToReset::parse_atr(atr);
        Ok(AnswerToReset {
            raw_atr: Some(atr.to_vec()),
            ss: ss,
            rid: Some(rid),
            card_name: Some(card_name),
        })
    }
    pub fn get_raw_atr(&self) -> Option<&Vec<u8>> {
        match self.raw_atr {
            Some(ref atr) => Some(atr),
            None => None,
        }
    }
    fn parse_atr(atr: &[u8; 32]) -> (Vec<u8>, u8, (String, CardName)) {
        let rid = atr[7..7 + 5].to_vec();
        let unknown_tag = "Unknown Tag Name";
        let ss = atr[12];
        let card_name = match (atr[13], atr[14]) {
            (0x00, v) => match v {
                0x01 => ("MIFARE Classic 1K", CardName::MifareClassic1k),
                0x02 => ("MIFARE Classic 4K", CardName::MifareClassic4k),
                0x03 => ("MIFARE Ultralight", CardName::MifareUltralight),
                0x07 => ("SRIX512", CardName::Srix512),
                0x26 => ("MIFARE Mini", CardName::MifareMini),
                0x36 => ("MIFARE Plus SL1 2K", CardName::MifarePlusSl12k),
                0x37 => ("MIFARE Plus SL1 4K", CardName::MifarePlusSl14k),
                0x38 => ("MIFARE Plus SL2 2K", CardName::MifarePlusSl12k),
                0x39 => ("MIFARE Plus SL2 4K", CardName::MifarePlusSl14k),
                0x3A => ("MIFARE Ultralight C", CardName::MifareUltralightC),
                0x30 => ("Topaz/Jewel", CardName::TopazJewel),
                0x3B => ("FeliCa", CardName::Felica),
                _ => (unknown_tag, CardName::UnknownTagName),
            },
            (0xFF, 28) => ("JCOP 30", CardName::Jcop30),
            _ => (unknown_tag, CardName::UnknownTagName),
        };
        (rid, ss, (card_name.0.to_owned(), card_name.1))
    }
    pub fn rid_to_string(&self) -> String {
        let mut s = self
            .rid
            .as_ref()
            .unwrap()
            .iter()
            .map(|b| format!("{:02x}-", b))
            .collect::<String>();
        s.pop();
        s
    }
}

impl Default for AnswerToReset {
    fn default() -> Self {
        AnswerToReset {
            raw_atr: None,
            ss: 0,
            rid: None,
            card_name: None,
        }
    }
}
impl std::fmt::Display for AnswerToReset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.card_name.as_ref().unwrap();
        write!(
            f,
            "ss: {}\nrid: {}\ncard_name: {}",
            self.ss,
            self.rid_to_string(),
            s.0
        )
    }
}

#[derive(Debug)]
pub enum ATRParseErrorCode {
    InvalidHeader(u8),
}

#[derive(Debug)]
pub struct ATRParseError {
    code: ATRParseErrorCode,
}

impl ATRParseError {
    pub fn new(code: ATRParseErrorCode) -> Self {
        ATRParseError { code: code }
    }
}
impl std::error::Error for ATRParseError {}
impl std::fmt::Display for ATRParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait ApduBuilderExtWithPcsc3V2 {
    fn get_serial(&mut self) -> &mut Self;
    fn get_ats(&mut self) -> &mut Self;
}

pub trait ApduBuilderExtWithFelica {
    fn get_card_id(&mut self) -> &mut Self;
    fn get_card_kind(&mut self) -> &mut Self;
    fn get_card_kind_name(&mut self) -> &mut Self;
    fn get_card_name(&mut self) -> &mut Self;
}

// マイナンバーカード拡張
pub trait JpkiExt {}

pub trait MifareExt {
    // MIFARE 1K/4K PICC
    fn load_auth_keys(
        reserved_key_struct: u8,
        key_no: u8,
    ) -> Result<(), Box<dyn std::error::Error>>;
    // MIFARE 1K/4K への認証
}
