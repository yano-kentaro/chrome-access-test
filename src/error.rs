#[derive(thiserror::Error, Debug)]
pub enum CustomError {
    #[error("指定したURLにアクセスできませんでした。")]
    AccessUrlError,

    #[error("不正なCookieが指定されています。")]
    CookieError,

    #[error("指定したセレクタが見つかりませんでした。")]
    FindSelectorError,
}
