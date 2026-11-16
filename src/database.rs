use crate::{Joke, User};

pub const DEFAULT_DATABASE_URL: &str = "sqlite:./data.db";

/// Read configuration shared by the server and migration CLI.
pub fn database_url() -> Result<String, std::env::VarError> {
    match std::env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(std::env::VarError::NotPresent) => Ok(DEFAULT_DATABASE_URL.to_owned()),
        Err(err) => Err(err),
    }
}

/// Connect with the same model registry in every executable and test.
pub async fn connect(url: &str) -> toasty::Result<toasty::Db> {
    toasty::Db::builder()
        .models(toasty::models!(Joke, User))
        .connect(url)
        .await
}
