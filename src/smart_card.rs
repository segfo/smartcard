// SmartCard Trait
pub trait APDU{

}

pub enum SmartcardConnectMethod{
    Default,ListIdx(usize),ID(String)
}
pub trait Smartcard{
    fn version_str(&self)->Option<&str>;
    fn version(&self)->Option<SmartcardVersion>;
    fn connect_reader(&self,con_method:SmartcardConnectMethod)->Result<(), SmartcardError>;
    /// コマンドの送信
    fn transmit(&self,data:Box<dyn APDU>);
    /// プロトコルを設定すると現在アクティブなプロトコルが返却される。
    /// 現在アクティブなプロトコルを知りたい場合や明示的に変更をしない場合は
    /// ProtocolType::InActive を使うと良い。
    fn config_protocol(&self,protocol:ProtocolType)->Option<ProtocolType>;
}
#[derive(Debug,Clone,Copy)]
pub struct SmartcardVersion{
    pub major:u32,
    pub minor:u32,
    pub build:u32,
    pub revision:u32,
}

impl SmartcardVersion{
    pub fn new(major:u32,minor:u32,build:u32,revision:u32)->Self{
        SmartcardVersion{
            major:major,
            minor:minor,
            build:build,
            revision:revision
        }
    }
}

pub enum ProtocolType{
    /// 現在アクティブなプロトコルをそのまま使う
    InActive,
    /// Raw Transferプロトコル（多分設定されない気はするが一応） 
    RAW,
    /// ISO 7816/3 T=0プロトコル：半二重非同期通信キャラクタベース
    T0,
    /// ISO 7816/3 T=1プロトコル：半二重非同期通信ブロック（バイナリ）ベース
    T1,
}

pub enum SmartcardState{
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
    UndefineState(Option<String>)

}

#[derive(Debug,Clone,Copy)]
pub enum SmartcardErrorKind{
    /// 成功
    Success,
    /// PC/SCリソースマネージャの接続コンテキスト初期化時のエラー
    /// 大抵はリソースマネージャサービスが立ち上がってない場合に発生するはず
    ResMgrCtxInit,
    /// リーダが使えない、接続されていない場合
    ReaderNotAvailable,
    /// リーダを検出できなかった場合
    ReaderDetectionFailed,
    /// スマートカードとの接続が途中で切れたときのエラー
    ConnectionLost,
}

#[derive(Debug)]
pub struct SmartcardError{
    /// エラー文字列、エラー発生時のハンドラがエラー種別に応じて自動的に設定する
    msg:String,
    /// エラー種別
    kind:SmartcardErrorKind
}

impl SmartcardError{
    pub fn new(kind:SmartcardErrorKind)->Self{
        SmartcardError{
            msg:SmartcardError::kind2msg(kind),
            kind:kind
        }
    }
    fn kind2msg(kind:SmartcardErrorKind)->String{
        let msg = match kind{
            SmartcardErrorKind::Success=>"Success",
            SmartcardErrorKind::ResMgrCtxInit=>"Resource manager unavailable. context acquire failed.",
            SmartcardErrorKind::ReaderDetectionFailed=>"Smart card reader detection failed.",
            SmartcardErrorKind::ReaderNotAvailable=>"Smart card reader not available.",
            SmartcardErrorKind::ConnectionLost=>"The connection to the smart card has been lost.",
        };
        msg.to_owned()
    }
}

impl std::fmt::Display for SmartcardError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",self.msg)
    }
}

impl std::error::Error for SmartcardError{}
