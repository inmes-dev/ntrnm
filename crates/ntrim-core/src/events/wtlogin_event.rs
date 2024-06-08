

#[derive(Debug)]
pub enum WtloginResponse {
    /// Login success.
    Success(),
    /// WtLogin failed.
    Fail(anyhow::Error),
    /// Refresh Sig Success.
    RefreshSigSuccess,
}