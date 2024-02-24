#![warn(clippy::all, clippy::pedantic)]

use async_mutex::Mutex;
use futures::executor::block_on;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashSet, env};

use dotenv::dotenv;
use hackthebot::htb::{api::new_htbapi_instance, api_types::HTBAPIConfig};
use hackthebot::{load_solves_to_cache, ScheduleRunnerData};
use serenity::async_trait;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::id::ChannelId;
use serenity::prelude::GatewayIntents;
use serenity::{http::Http, model::id::UserId, Client};
use serenity::{model::gateway::Ready, model::Permissions};
use tasks::{process_new_solves, process_rank_status};

mod discord_utils;
mod tasks;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected.", ready.user.name);

        match ready.user.invite_url(&ctx.http, Permissions::empty()).await {
            Ok(url) => {
                log::info!("Invite me using this url: {}", &url);
            }
            Err(why) => {
                log::error!("Error getting invite url: {why:?}");
            }
        };
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "hackthebot=INFO");
    pretty_env_logger::init_timed();

    dotenv().ok();
    color_eyre::install().expect("Error when setting up color_eyre");

    log::info!("Reading environment variables...");

    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in your environment (DISCORD_TOKEN)");
    let owner_id_str = env::var("OWNER_ID").expect("Expected an OWNER_ID in your environment!");
    let application_id =
        env::var("APPLICATION_ID").expect("Expected an APPLICATION_ID in your environment!");
    let application_id = application_id
        .parse::<u64>()
        .expect("Could not parse APPLICATION_ID!");
    let owner_id = owner_id_str
        .parse::<u64>()
        .expect("Unable to parse OWNER_ID into u64... Did you put it in correctly?");
    let team_id = env::var("HTB_TEAM_ID")
        .expect("No HTB_TEAM_ID environment variable found!")
        .parse::<i32>()
        .expect("HTB_TEAM_ID isn't a number!");
    let email = env::var("HTB_EMAIL").expect("No HTB_EMAIL environment variable found!");
    let pass = env::var("HTB_PASSWORD").expect("No HTB_PASSWORD environment variable found!");
    let htb_channel_id = env::var("HTB_CHANNEL_ID")
        .expect("No HTB_CHANNEL_ID environment variable found!")
        .parse::<u64>()
        .expect("HTB_CHANNEL_ID environment variable was unable to be parsed to a u64...");

    let mut owners = HashSet::new();
    owners.insert(UserId(owner_id));

    log::info!("Setting up discord client...");

    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let htb_config = HTBAPIConfig {
        email,
        password: pass,
        team_id,
    };

    log::info!("Initialising HTB API instance...");

    let htb_api = new_htbapi_instance(htb_config)
        .await
        .expect("Error when creating HTBApi instance...");
    let http = Http::new_with_application_id(&token, application_id);
    let channel_id = ChannelId(htb_channel_id);

    log::info!("Building scheduler data...");

    let scheduler_data = ScheduleRunnerData {
        htb_api,
        http,
        channel_id,
    };

    let threadsafe_data = Arc::new(Mutex::new(scheduler_data));

    // Load the current solves into memory, which will be used for diffing later.
    let data = threadsafe_data.lock().await;
    if let Err(why) = load_solves_to_cache(&data.htb_api).await {
        log::error!("Error loading categories to cache... {why}");
    }
    std::mem::drop(data);

    let data_arc1 = threadsafe_data.clone();
    let data_arc2 = threadsafe_data.clone();

    std::thread::spawn(move || loop {
        let guard = block_on(data_arc1.lock());

        log::info!("Processing current rank...");
        if let Err(why) = process_rank_status(guard) {
            log::error!("Error updating team rank status: {why:?}");
        }

        // Sleep for a day.
        std::thread::sleep(Duration::from_secs(86400));
    });

    std::thread::spawn(move || loop {
        let guard = block_on(data_arc2.lock());

        match process_new_solves(guard) {
            Ok(num_new_solves) => {
                log::info!("Successfully processed {num_new_solves} solves.");
            }
            Err(why) => {
                log::error!("Error processing new HTB solves: {why:?}");
            }
        }

        // Sleep for 1 minute.
        std::thread::sleep(Duration::from_secs(60));
    });

    if let Err(why) = client.start().await {
        log::error!("Client error: {why:?}");
    }
}
