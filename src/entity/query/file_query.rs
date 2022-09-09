use serde::{Deserialize};
#[derive(Deserialize)]
pub struct RemoveFileQuery{
    pub path:String,
    pub is_file:bool,
}