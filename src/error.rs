#[derive(thiserror::Error, Debug)]
pub enum CustomError {
    #[error("アクセス確認先のURLが指定されていません。")]
    AccessUrlNotDefined,

    #[error("アクセス確認先のセレクタが指定されていません。")]
    FindSelectorNotDefined,

    #[error("指定したURLにアクセスできませんでした。")]
    AccessUrlError,

    #[error("不正なCookieが指定されています。")]
    CookieError,

    #[error("指定したセレクタが見つかりませんでした。")]
    FindSelectorError,
}
