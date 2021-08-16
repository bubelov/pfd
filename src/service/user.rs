use crate::{model::User, repository::UserRepository};
use anyhow::Result;

pub fn insert(user: &User, repo: &UserRepository) -> Result<()> {
    repo.insert(&user)
}

pub fn select_by_username(username: &str, repo: &UserRepository) -> Result<Option<User>> {
    repo.select_by_username(&username)
}
