use crate::models::Joke;
use toasty::Result;

pub async fn update(
    id: i64,
    content: &str,
    db: &mut toasty::Db,
) -> Result<Option<Joke>, toasty::Error> {
    let existing = Joke::filter_by_id(id).first().exec(db).await?;
    if let Some(mut j) = existing {
        toasty::update!(j { content }).exec(db).await?;
        Ok(Some(j))
    } else {
        Ok(None)
    }
}
