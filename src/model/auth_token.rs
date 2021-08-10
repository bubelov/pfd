use crate::model::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthToken {
    pub id: Id,
    pub username: String,
}
