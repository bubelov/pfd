use crate::{model::User, repository::UserRepository};
use anyhow::Result;

pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(repo: &UserRepository) -> UserService {
        UserService { repo: repo.clone() }
    }

    pub fn insert(&self, user: &User) -> Result<()> {
        self.repo.insert(&user)
    }

    pub fn select_by_username(&self, username: &str) -> Result<Option<User>> {
        self.repo.select_by_username(&username)
    }
}
