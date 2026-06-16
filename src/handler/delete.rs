use crate::models::Joke;
use toasty::Result;

pub async fn remove(db: &mut toasty::Db) -> Result<()> {
    Joke::all().delete().exec(db).await
}

pub async fn delete_joke(id: i64, db: &mut toasty::Db) -> Result<()> {
    Joke::delete_by_id(db, id).await
}
