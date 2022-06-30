use async_mutex::MutexGuard;
use color_eyre::eyre::Error;
use hackthebot::{
    get_hasura_client, graphql::graphql_provider::GraphQLProvider, htb::api_types::HTBApiClient,
    ScheduleRunnerData, HASURA_PROVIDER,
};

use hackthebot::update_htb_channel_topic_with_stats;

use crate::discord_utils::{announce_solve, SolveToAnnounce};

#[tokio::main]
pub async fn update_htb_challenges(htb_api: &HTBApiClient) -> Result<(), Error> {
    let challenges = htb_api.list_active_challenges().await?;
    let hasura_provider = HASURA_PROVIDER.get_or_init(get_hasura_client).await;

    // Process all challenges
    for challenge in challenges.challenges {
        // Check the challenge doesn't already exist.
        let db_challenges = hasura_provider.get_challenge_by_id(challenge.id).await?;
        if db_challenges.is_empty() {
            println!("Found a new challenge! Adding now... {:#?}", challenge);
            // Insert the challenge.
            hasura_provider
                .insert_challenge(
                    challenge.release_date.to_string(),
                    challenge.points.parse::<i64>()?,
                    challenge.name,
                    challenge.machine_avatar,
                    challenge.difficulty,
                    challenge.challenge_category_id,
                    challenge.id,
                )
                .await?;
        }
    }

    // Process all machines
    let machines = htb_api.list_active_machines().await?.info;
    for machine in machines {
        let db_challenges = hasura_provider.get_challenge_by_id(machine.id).await?;
        if db_challenges.is_empty() {
            println!("Found a new machine! Adding now... {:#?}", machine);
            // Insert the machine.
            hasura_provider
                .insert_challenge(
                    machine.release,
                    machine.points,
                    machine.name,
                    Some(machine.avatar),
                    machine.difficulty,
                    100,
                    machine.id,
                )
                .await?;
        }
    }

    Ok(())
}

#[tokio::main]
pub async fn process_rank_status(mut data: MutexGuard<ScheduleRunnerData>) -> Result<(), Error> {
    data.htb_api.handle_token_renewal().await?;
    let latest_rank = data.htb_api.get_team_rank().await?;

    if let Err(why) =
        update_htb_channel_topic_with_stats(&latest_rank.data, &data.channel_id, &data.http).await
    {
        eprintln!("Error when updating the HTB channel topic... {}", why);
    }

    Ok(())
}

#[tokio::main]
pub async fn process_new_solves(mut data: MutexGuard<ScheduleRunnerData>) -> Result<(), Error> {
    data.htb_api.handle_token_renewal().await?;
    let recent_team_solves = &data.htb_api.get_recent_team_activity().await?;
    let hasura_provider = HASURA_PROVIDER.get_or_init(get_hasura_client).await;

    for solve in recent_team_solves {
        let member = &solve.user;

        // If the challenge has already been solve by this user, we need to skip.
        if hasura_provider
            .is_challenge_solved_for_user(
                member.id,
                solve.id,
                solve.name.clone(),
                solve.solve_type.clone(),
            )
            .await?
        {
            continue;
        }

        let challenges = hasura_provider.get_challenge_by_id(solve.id).await?;
        if let Some(first_challenge) = challenges.first() {
            // Build the solve to announce
            let solve_to_announce = SolveToAnnounce {
                solver: member.name.clone(),
                user_id: member.id,
                solve_type: solve.solve_type.clone(),
                challenge: first_challenge,
            };

            // Announce the solve.
            match announce_solve(&solve_to_announce, &data.channel_id, &data.http).await {
                Ok(_) => {
                    // Now we store the solve in the database.
                    hasura_provider
                        .insert_solve_for_user(
                            member.id,
                            member.name.clone(),
                            solve.solve_type.clone(),
                            solve.id,
                        )
                        .await?;
                }
                Err(why) => {
                    eprintln!("Error when announcing solve... {}", why);
                }
            }
        }
    }

    Ok(())
}
