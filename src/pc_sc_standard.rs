// PC/SC規格に準拠したカードとの通信に関わる定義を記述していく

use crate::apdu_iso7816;

#[derive(Debug, Clone)]
pub struct AnswerToReset {
    pub raw_atr: Option<Vec<u8>>,
    pub historical_data: Option<Vec<u8>>, // historical data
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
    JpPublicTransitIc,
    JpMynaCard,
    JpDrivingLicenseCard,
    OtherTag,
    UnknownTagName
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
        let (historical_data, card_name) = AnswerToReset::parse_atr(atr);
        Ok(AnswerToReset {
            raw_atr: Some(atr.to_vec()),
            historical_data: Some(historical_data),
            card_name: Some(card_name),
        })
    }
    pub fn get_raw_atr(&self) -> Option<&Vec<u8>> {
        match self.raw_atr {
            Some(ref atr) => Some(atr),
            None => None,
        }
    }
    const unknown_tag: &str = "Unknown Tag Name";
    fn next_tables(atr: u8) -> (bool, usize) {
        let flags = atr >> 0x04 & 0x0f;
        let mut count = 0;
        for i in 0..4 {
            if (flags >> i) & 1 == 1 {
                count += 1;
            }
        }
        (flags & 0x8 == 0x08, count)
    }
    fn parse_atr(atr: &[u8; 32]) -> (Vec<u8>, (String, CardName)) {
        let historical_bytes = (atr[1] & 0x0f) as usize;
        // 真面目にT0をパースしないとridは取得できない。
        // カード種別を探索するために必須なのでRIDとる。
        let mut index = 1;
        loop {
            let (next_exists, count) = AnswerToReset::next_tables(atr[index]);
            index += count;
            if !next_exists {
                index += 1;
                break;
            }
        }
        let historical_data = atr[index..index + historical_bytes].to_vec();
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
                0x3B => AnswerToReset::lookup_to_histdata(historical_data.clone()),//("FeliCa", CardName::Felica),
                _ => AnswerToReset::lookup_to_histdata(historical_data.clone()),
            },
            (0xFF, 28) => ("JCOP 30", CardName::Jcop30),
            _ => (Self::unknown_tag, CardName::UnknownTagName),
        };
        (historical_data, (card_name.0.to_owned(), card_name.1))
    }
    const LOOKUP_TABLE:&'static[(&'static[u8],&'static str,CardName)] = &[
        (&[0x00,0x00,0x00,0x00,0x91,0x81,0xc1,0x00],"Driving License card of Japan",CardName::JpDrivingLicenseCard),
        (&[0x00,0x00,0x41,0xe0,0xb3,0x81,0xa1,0x00],"ID card issued by Japan government(aka My-number card)",CardName::JpMynaCard),
        (&[0x00,0x00,0x05,0xE0,0xB3,0x81,0xA1,0x00],"Japanese JPKI card (aka JINC card)",CardName::JpMynaCard),
        (&[0x80,0x4f,0x0c,0xa0,0x00,0x00,0x03,0x06,0x11,0x00,0x3b,0x00,0x00,0x00,0x00],"public transit card (Japan IC System)",CardName::JpPublicTransitIc)
    ];
    fn lookup_to_histdata(histrical_data: Vec<u8>) -> (&'static str, CardName) {
        for search in Self::LOOKUP_TABLE{
            if histrical_data == search.0{
                return (search.1,search.2.clone())
            }
        }
        (Self::unknown_tag, CardName::UnknownTagName)
    }
    pub fn historical_data_to_string(&self) -> String {
        let mut s = self
            .historical_data
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
            historical_data: None,
            card_name: None,
        }
    }
}
impl std::fmt::Display for AnswerToReset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.card_name.as_ref().unwrap();
        write!(
            f,
            "rid: {}\ncard_name: {}",
            self.historical_data_to_string(),
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

#[test]
fn atr_parse1(){
    let atr=[0x3B,0x88,0x8E,0xFE,0x53,0x2A,0x03,0x1E,0x04,0x92,0x80,0x00,0x41,0x32,0x36,0x01,0x11,0xDF];
    let mut index=1;
    loop {
        let (next_exists, count) = AnswerToReset::next_tables(atr[index]);
        index += count;
        if !next_exists {
            index += 1;
            break;
        }
    }
    assert_eq!(index,9);
}