#![allow(clippy::too_many_arguments)]

use anyhow::Error;
use async_trait::async_trait;

use super::graphql_types::*;

#[async_trait]
pub trait GraphQLProvider {
    async fn get_users_who_solved_challenge(&self, challenge_id: i64)
        -> Result<Vec<String>, Error>;
    async fn insert_solve_for_user(
        &self,
        user_id: i64,
        user_name: String,
        solve_type: String,
        challenge_id: i64,
    ) -> Result<(), Error>;
    async fn insert_challenge(
        &self,
        release_date: String,
        points: i64,
        name: String,
        machine_avatar: Option<String>,
        difficulty: String,
        category: i64,
        htb_id: i64,
    ) -> Result<(), Error>;
    async fn get_challenges_for_name(&self, name: &str) -> Result<Vec<Challenge>, Error>;
    async fn get_challenge_by_id(&self, htb_id: i64) -> Result<Vec<Challenge>, Error>;
    async fn is_challenge_solved_for_user(
        &self,
        user_id: i64,
        challenge_id: i64,
        challenge_name: String,
    ) -> Result<bool, Error>;
    async fn get_solves_for_user_by_name(&self, user_name: String) -> Result<Vec<Solve>, Error>;
}
