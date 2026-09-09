use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data.db".to_string());

    let db = toasty::Db::builder()
        .models(toasty::models!(axum_everyone::Joke, axum_everyone::User))
        .connect(&db_url)
        .await?;

    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;

    Ok(())
}
