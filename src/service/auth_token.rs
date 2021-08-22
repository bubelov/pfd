use crate::{
    model::{AuthToken, Id},
    repository::AuthTokenRepository,
};
use anyhow::Result;

pub struct AuthTokenService {
    repo: AuthTokenRepository,
}

impl AuthTokenService {
    pub fn new(repo: &AuthTokenRepository) -> AuthTokenService {
        AuthTokenService { repo: repo.clone() }
    }

    pub fn insert(&self, item: &AuthToken) -> Result<()> {
        self.repo.insert(item)
    }

    pub fn select_by_id(&self, id: &Id) -> Result<Option<AuthToken>> {
        self.repo.select_by_id(id)
    }
}
