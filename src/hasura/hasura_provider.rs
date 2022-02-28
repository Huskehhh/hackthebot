use anyhow::{Error, Result};
use async_trait::async_trait;
use graphql_client::reqwest::post_graphql;

use crate::graphql::{graphql_provider::GraphQLProvider, graphql_types::*};

#[derive(Debug)]
pub struct HasuraProvider {
    pub hasura_client: reqwest::Client,
    pub hasura_api_url: String,
}

#[async_trait]
impl GraphQLProvider for HasuraProvider {
    async fn get_users_who_solved_challenge(
        &self,
        challenge_id: i64,
    ) -> Result<Vec<String>, Error> {
        let mut solvers = vec![];
        let variables = get_users_who_solved_challenge::Variables { challenge_id };

        let response_body = post_graphql::<GetUsersWhoSolvedChallenge, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Getting users who have solved challenge");
        }

        if let Some(solving_users) = response_body.data {
            solving_users.htb_solves.into_iter().for_each(|solve| {
                solvers.push(solve.user_name);
            });
        }

        Ok(solvers)
    }

    async fn insert_solve_for_user(
        &self,
        user_id: i64,
        user_name: String,
        solve_type: String,
        challenge_id: i64,
    ) -> Result<(), Error> {
        let variables = insert_solve_for_user::Variables {
            user_name,
            user_id,
            solve_type,
            challenge_id,
        };

        let response_body = post_graphql::<InsertSolveForUser, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Inserting solve for user");
        }

        Ok(())
    }

    async fn insert_challenge(
        &self,
        release_date: String,
        points: i64,
        name: String,
        machine_avatar: Option<String>,
        difficulty: String,
        category: i64,
        htb_id: i64,
    ) -> Result<(), Error> {
        let variables = insert_challenge::Variables {
            release_date,
            points,
            name,
            machine_avatar,
            difficulty,
            category,
            htb_id,
        };

        let response_body = post_graphql::<InsertChallenge, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Inserting challenge data");
        }

        Ok(())
    }

    async fn get_challenges_for_name(&self, name: &str) -> Result<Vec<Challenge>, Error> {
        // We need to add % to each side to indicate a wildcard search.
        let name_query = format!("%{}%", name);
        let variables = get_challenge_by_name::Variables { name: name_query };

        let response_body = post_graphql::<GetChallengeByName, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Getting challenge by name");
        }

        match response_body.data {
            Some(challenges) => Ok(challenges
                .htb_challenges
                .into_iter()
                .map(Challenge::from)
                .collect()),
            None => Ok(vec![]),
        }
    }

    async fn get_challenge_by_id(&self, htb_id: i64) -> Result<Vec<Challenge>, Error> {
        let variables = get_challenge_by_id::Variables { htb_id };

        let response_body = post_graphql::<GetChallengeById, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Getting challenge by name");
        }

        match response_body.data {
            Some(challenges) => Ok(challenges
                .htb_challenges
                .into_iter()
                .map(Challenge::from)
                .collect()),
            None => Ok(vec![]),
        }
    }

    async fn is_challenge_solved_for_user(
        &self,
        user_id: i64,
        challenge_id: i64,
        challenge_name: String,
        solve_type: String,
    ) -> Result<bool, Error> {
        let variables = is_challenge_solved_for_user::Variables {
            user_id,
            challenge_id,
            challenge_name,
            solve_type,
        };

        let response_body = post_graphql::<IsChallengeSolvedForUser, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Checking if challenge was solved for user");
        }

        if let Some(challenge) = response_body.data {
            if !challenge.htb_solves.is_empty() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn get_solves_for_user_by_name(&self, user_name: String) -> Result<Vec<Solve>, Error> {
        let variables = get_solves_for_user_with_name::Variables { user_name };

        let response_body = post_graphql::<GetSolvesForUserWithName, _>(
            &self.hasura_client,
            &self.hasura_api_url,
            variables,
        )
        .await?;

        // Print the GraphQL errors, if they exist.
        if let Some(why) = response_body.errors {
            print_graphql_errors(&why, "Checking if challenge was solved for user");
        }

        match response_body.data {
            Some(challenge) => Ok(challenge.htb_solves),
            None => Ok(vec![]),
        }
    }
}

// Print the GraphQL errors, if they exist.
fn print_graphql_errors(errors: &[graphql_client::Error], operation: &str) {
    errors.iter().for_each(|why| {
        eprintln!("Error when {}... why: {}", operation, why);
    });
}
