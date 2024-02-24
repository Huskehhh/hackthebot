use std::time::Duration;

use chrono::{DateTime, Local};
use color_eyre::eyre::{eyre, Error, Result};
use dashmap::{DashMap, DashSet};
use htb::api_types::*;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serenity::{http::Http, model::id::ChannelId};

pub mod htb;

pub static SOLVE_CACHE: Lazy<DashMap<i64, DashSet<RecentTeamSolve>>> = Lazy::new(DashMap::new);

#[derive(Debug)]
pub struct Challenge {
    pub name: String,
    pub points: i64,
    pub challenge_type: String,
    pub machine_avatar: Option<String>,
    pub challenge_category: Option<String>,
}

#[derive(Debug)]
pub struct ScheduleRunnerData {
    pub htb_api: HTBApiClient,
    pub http: Http,
    pub channel_id: ChannelId,
}

pub async fn load_solves_to_cache(htb_api: &HTBApiClient) -> Result<()> {
    let team_solves = htb_api.get_recent_team_activity().await?;

    for solve in team_solves {
        match SOLVE_CACHE.get_mut(&solve.user.id) {
            Some(solves) => {
                solves.insert(solve);
            }
            None => {
                let solves = DashSet::new();
                let user_id = solve.user.id;
                solves.insert(solve);
                SOLVE_CACHE.insert(user_id, solves);
            }
        };
    }

    Ok(())
}

pub async fn update_htb_channel_topic_with_stats(
    stats: &RankStatsData,
    channel_id: &ChannelId,
    http: &Http,
) -> Result<(), Error> {
    let local_timestamp: DateTime<Local> = Local::now();
    let timestamp_string = local_timestamp.format("%a %b %e %T").to_string();

    let new_channel_topic = format!(
        "Team rank {}, Points: {}. Last updated: {}",
        stats.rank, stats.points, timestamp_string
    );

    match channel_id.edit(&http, |c| c.topic(new_channel_topic)).await {
        Ok(_) => Ok(()),
        Err(why) => Err(eyre!("Error when updating channel topic: {}", why)),
    }
}

pub fn jwt_still_valid(jwt: &JWTClaims) -> bool {
    let local: DateTime<Local> = Local::now();
    let unix_epoch = local.timestamp() as f64;

    if unix_epoch > jwt.exp {
        return false;
    }

    true
}

pub fn create_reqwest_client(api_key: &str, token_type: &str) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();

    let auth_header = HeaderValue::from_str(&format!("{} {}", token_type, &api_key))?;

    let content_type_header = HeaderValue::from_str("application/json")?;

    headers.insert("Authorization", auth_header);
    headers.insert("Content-Type", content_type_header);

    Ok(ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .cookie_store(true)
        .default_headers(headers)
        .build()?)
}
