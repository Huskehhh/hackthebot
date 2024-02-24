use async_mutex::MutexGuard;
use color_eyre::eyre::Error;
use hackthebot::Challenge;
use hackthebot::ScheduleRunnerData;

use hackthebot::update_htb_channel_topic_with_stats;
use hackthebot::SOLVE_CACHE;

use crate::discord_utils::{announce_solve, SolveToAnnounce};

#[tokio::main]
pub async fn process_rank_status(mut data: MutexGuard<ScheduleRunnerData>) -> Result<(), Error> {
    data.htb_api.handle_token_renewal().await?;
    let latest_rank = data.htb_api.get_team_rank().await?;

    if let Err(why) =
        update_htb_channel_topic_with_stats(&latest_rank.data, &data.channel_id, &data.http).await
    {
        log::error!("Error when updating the HTB channel topic... {why}");
    }

    Ok(())
}

#[tokio::main]
pub async fn process_new_solves(mut data: MutexGuard<ScheduleRunnerData>) -> Result<usize, Error> {
    data.htb_api.handle_token_renewal().await?;
    let team_activity = data.htb_api.get_recent_team_activity().await?;
    let mut num_new_solves = 0;

    for solve in team_activity {
        let solver_id = solve.user.id;

        if let Some(previous_solves) = SOLVE_CACHE.get_mut(&solver_id) {
            if previous_solves.contains(&solve) {
                continue;
            }

            let challenge = Challenge {
                name: solve.name.clone(),
                machine_avatar: solve.machine_avatar.clone(),
                points: solve.points,
                challenge_type: solve.object_type.clone(),
                challenge_category: solve.challenge_category.clone(),
            };

            let announce = SolveToAnnounce {
                solver: solve.user.name.clone(),
                user_id: solver_id,
                solve_type: solve.solve_type.clone(),
                challenge,
            };

            match announce_solve(&announce, &data.channel_id, &data.http).await {
                Ok(()) => {
                    previous_solves.insert(solve);
                    num_new_solves += 1;
                }
                Err(why) => {
                    log::error!("Error when announcing solve {announce:#?}, err: {why}");
                }
            }
        }
    }

    Ok(num_new_solves)
}
