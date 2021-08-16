use crate::{
    model::{AuthToken, Id},
    repository::AuthTokenRepository,
};
use anyhow::Result;

pub fn insert(item: &AuthToken, repo: &AuthTokenRepository) -> Result<()> {
    repo.insert(item)
}

pub fn select_by_id(id: &Id, repo: &AuthTokenRepository) -> Result<Option<AuthToken>> {
    repo.select_by_id(id)
}
