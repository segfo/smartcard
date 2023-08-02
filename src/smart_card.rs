// SmartCardの抽象実装（Trait）

use crate::pc_sc_standard::AnswerToReset;

// APDUは実装依存のためTraitにしておく
pub trait APDU {
    fn read8(&self) -> Vec<u8>;
}

/// 利用するときに、固定するのかユーザに選ばせるのか
pub enum SmartcardConnectMethod {
    UserPrompt,
    ListIdx(usize),
}
pub trait Smartcard {
    fn version_str(&self) -> Option<String>;
    fn version(&self) -> Option<SmartcardVersion>;
    fn connect_reader(
        &mut self,
        con_method: SmartcardConnectMethod,
    ) -> Result<ProtocolType, SmartcardError>;
    /// コマンドの送信
    fn transmit(&self, data: Box<dyn APDU>) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    /// プロトコルを設定すると現在アクティブなプロトコルが返却される。
    /// 現在アクティブなプロトコルを知りたい場合や明示的に変更をしない場合は
    /// ProtocolType::InActive を使うと良い。
    fn config_protocol(&mut self, protocol: ProtocolType) -> Option<ProtocolType>;
    fn get_atr(&self)->&AnswerToReset;
}

pub trait SmartcardInfo{
    fn get_card_type()->Option<(u32,String)>;
    fn get_card_name()->Option<(u32,String)>;
    fn get_card_kind()->Option<(u32,String)>;
}

// pub trait Apdu_Iso7816{
//     fn
// }

#[derive(Debug, Clone, Copy)]
pub struct SmartcardVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub revision: u32,
}

impl SmartcardVersion {
    pub fn new(major: u32, minor: u32, build: u32, revision: u32) -> Self {
        SmartcardVersion {
            major: major,
            minor: minor,
            build: build,
            revision: revision,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolType {
    /// 現在アクティブなプロトコルをそのまま使う
    InActive,
    /// Raw Transferプロトコル（多分設定されない気はするが一応）
    RAW,
    /// ISO 7816/3 T=0プロトコル：半二重非同期通信キャラクタベース
    T0,
    /// ISO 7816/3 T=1プロトコル：半二重非同期通信ブロック（バイナリ）ベース
    T1,
    // 上記2つのプロトコルの組み合わせ
    T0T1,
    // 上記以外の不明なプロトコルの場合
    Unknown,
}
impl ToString for ProtocolType{
    fn to_string(&self) -> String {
        match self {
            ProtocolType::T1 => "T1(ブロック転送)",
            ProtocolType::T0 => "T0(キャラクタ転送)",
            ProtocolType::RAW => "RAW",
            _ => "<不明>",
        }.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SmartcardState {
    /// スキャン開始
    /// 内包パラメータはタイムアウト時間、Noneでタイムアウトなし（無限に待機）
    Scanning(Option<usize>),
    /// タイムアウト
    ScanTimeout,
    /// カードが挿入またはセットされていない
    CardReaderEmpty,
    /// カードリーダが接続されていない
    CardReaderUnavailable,
    /// その他、未定義なエラー
    /// ユーザ通知用のエラー文字列があれば設定する
    UndefineState(Option<String>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmartcardErrorKind {
    /// 成功
    Success,
    /// PC/SCリソースマネージャの接続コンテキスト初期化時のエラー
    /// 大抵はリソースマネージャサービスが立ち上がってない場合に発生するはず
    ResMgrCtxInit,
    /// リーダが使えない、接続されていない場合
    ReaderNotAvailable,
    /// カードが使えなかった場合
    CardNotAvailable,
    /// カードを検出できなかった場合
    CardDetectionFailed,
    /// リーダを検出できなかった場合
    ReaderDetectionFailed,
    /// スマートカードに接続ができなかった場合のエラー
    NotReady,
    /// プロトコルに互換性がない
    ProtocolMismatch,
    /// スマートカードとの接続が途中で切れたときのエラー
    ConnectionLost,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmartcardError {
    /// エラー文字列、エラー発生時のハンドラがエラー種別に応じて自動的に設定する
    msg: String,
    /// エラー種別
    kind: SmartcardErrorKind,
}

impl SmartcardError {
    pub fn new(kind: SmartcardErrorKind) -> Self {
        SmartcardError {
            msg: SmartcardError::kind2msg(kind),
            kind: kind,
        }
    }
    fn kind2msg(kind: SmartcardErrorKind) -> String {
        let msg = match kind {
            SmartcardErrorKind::Success => "Success",
            SmartcardErrorKind::ResMgrCtxInit => {
                "Resource manager unavailable. context acquire failed."
            }
            SmartcardErrorKind::ReaderDetectionFailed => "Smart card reader detection failed.",
            SmartcardErrorKind::ReaderNotAvailable => "Smart card reader not available.",
            SmartcardErrorKind::ConnectionLost => "The connection to the smart card has been lost.",
            SmartcardErrorKind::CardNotAvailable => "Smart card not available.",
            SmartcardErrorKind::CardDetectionFailed => "Smart card detection failed.",
            SmartcardErrorKind::ProtocolMismatch => "Smart card Protocol mismatch.",
            SmartcardErrorKind::NotReady => "Smart card not ready.",
        };
        msg.to_owned()
    }
}

impl std::fmt::Display for SmartcardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for SmartcardError {}
