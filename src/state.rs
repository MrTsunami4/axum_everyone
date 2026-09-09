/// Application state shared across all handlers.
/// Contains the Toasty database handle.
#[derive(Clone)]
pub struct AppState {
    pub db: toasty::Db,
}
