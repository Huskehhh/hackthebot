use graphql_client::GraphQLQuery;

use self::{
    get_challenge_by_id::GetChallengeByIdHtbChallenges,
    get_challenge_by_name::GetChallengeByNameHtbChallenges,
    get_solves_for_user_with_name::GetSolvesForUserWithNameHtbSolves,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/GetChallengeByName.graphql",
    response_derives = "Debug"
)]
pub struct GetChallengeByName;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/GetChallengeById.graphql",
    response_derives = "Debug"
)]
pub struct GetChallengeById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/GetUsersWhoSolvedChallenge.graphql",
    response_derives = "Debug"
)]
pub struct GetUsersWhoSolvedChallenge;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/InsertChallenge.graphql",
    response_derives = "Debug"
)]
pub struct InsertChallenge;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/InsertSolveForUser.graphql",
    response_derives = "Debug"
)]
pub struct InsertSolveForUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/IsChallengeSolvedForUser.graphql",
    response_derives = "Debug"
)]
pub struct IsChallengeSolvedForUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/GetSolvesForUserWithName.graphql",
    response_derives = "Debug"
)]
pub struct GetSolvesForUserWithName;

// Redeclared types

pub type Solve = GetSolvesForUserWithNameHtbSolves;

#[derive(Debug)]
pub struct Challenge {
    pub id: i64,
    pub htb_id: i64,
    pub name: String,
    pub difficulty: String,
    pub points: i64,
    pub release_date: String,
    pub category: i64,
    pub machine_avatar: Option<String>,
}

impl From<GetChallengeByIdHtbChallenges> for Challenge {
    fn from(challenge: GetChallengeByIdHtbChallenges) -> Self {
        Challenge {
            id: challenge.id,
            htb_id: challenge.htb_id,
            name: challenge.name,
            difficulty: challenge.difficulty,
            points: challenge.points,
            release_date: challenge.release_date,
            category: challenge.category,
            machine_avatar: challenge.machine_avatar,
        }
    }
}

impl From<GetChallengeByNameHtbChallenges> for Challenge {
    fn from(challenge: GetChallengeByNameHtbChallenges) -> Self {
        Challenge {
            id: challenge.id,
            htb_id: challenge.htb_id,
            name: challenge.name,
            difficulty: challenge.difficulty,
            points: challenge.points,
            release_date: challenge.release_date,
            category: challenge.category,
            machine_avatar: challenge.machine_avatar,
        }
    }
}
