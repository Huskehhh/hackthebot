use std::{env, time::Duration};

use chrono::{DateTime, Local};
use color_eyre::eyre::{eyre, Error, Result};
use dashmap::DashMap;
use hasura::hasura_provider::HasuraProvider;
use htb::api_types::*;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serenity::{http::Http, model::id::ChannelId};
use tokio::sync::OnceCell;

pub mod graphql;
mod hasura;
pub mod htb;

pub static CATEGORY_CACHE: Lazy<DashMap<i64, String>> = Lazy::new(DashMap::new);
pub static HASURA_PROVIDER: OnceCell<HasuraProvider> = OnceCell::const_new();

#[derive(Debug)]
pub struct ScheduleRunnerData {
    pub htb_api: HTBApiClient,
    pub http: Http,
    pub channel_id: ChannelId,
}

pub async fn get_hasura_client() -> HasuraProvider {
    let hasura_api_key =
        env::var("HASURA_API_KEY").expect("No HASURA_API_KEY environment variable defined!");
    let hasura_api_url =
        env::var("HASURA_API_URL").expect("No HASURA_API_URL environment variable defined!");

    let hasura_client = create_hasura_reqwest_client(&hasura_api_key).unwrap();

    HasuraProvider {
        hasura_client,
        hasura_api_url,
    }
}

pub async fn load_categories_to_cache(htb_api: &HTBApiClient) -> Result<(), Error> {
    let challenge_categories_response = htb_api.get_challenge_categories().await?;

    for category in challenge_categories_response.info {
        CATEGORY_CACHE.insert(category.id, category.name);
    }

    // Machines don't have a category, so we add one with an ID that wont collide
    CATEGORY_CACHE.insert(100, "Machine".to_owned());

    Ok(())
}

pub fn get_challenge_category_from_id(challenge_category_id: i64) -> String {
    match CATEGORY_CACHE.get(&challenge_category_id) {
        Some(cached) => cached.value().clone(),
        None => "Unknown".to_string(),
    }
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
    let unix_epoch = local.timestamp();

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

pub fn create_hasura_reqwest_client(hasura_api_key: &str) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();

    let auth_header = HeaderValue::from_str(hasura_api_key)?;

    let content_type_header = HeaderValue::from_str("application/json")?;

    headers.insert("x-hasura-admin-secret", auth_header);
    headers.insert("Content-Type", content_type_header);

    Ok(ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()?)
}
