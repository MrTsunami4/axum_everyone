use crate::models::Joke;
use toasty::Result;

pub async fn add(content: &str, db: &mut toasty::Db) -> Result<Joke> {
    toasty::create!(Joke { content }).exec(db).await
}
