use crate::models::Joke;
use toasty::Result;

pub async fn get_joke(id: i64, db: &mut toasty::Db) -> Result<Option<Joke>> {
    Joke::filter_by_id(id).first().exec(db).await
}

pub async fn get_all_jokes(db: &mut toasty::Db) -> Result<Vec<Joke>> {
    Joke::all()
        .order_by(Joke::fields().id().asc())
        .exec(db)
        .await
}
