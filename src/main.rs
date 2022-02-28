use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{collections::HashSet, env};

use commands::HTBER_GROUP;
use dotenv::dotenv;
use hackthebot::htb::{api::new_htbapi_instance, api_types::HTBAPIConfig};
use hackthebot::{load_categories_to_cache, ScheduleRunnerData};
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::{
    client::Context,
    framework::standard::{help_commands, Args, CommandGroup, CommandResult, HelpOptions},
};
use serenity::{client::EventHandler, model::prelude::Activity};
use serenity::{
    framework::standard::{macros::*, DispatchError},
    model::id::ChannelId,
};
use serenity::{http::Http, model::id::UserId, Client};
use serenity::{model::gateway::Ready, model::Permissions};
use tasks::{process_new_solves, process_rank_status, update_htb_challenges};

mod commands;
mod discord_utils;
mod tasks;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ctx.set_activity(Activity::playing(".help for more info!"))
            .await;

        match ready.user.invite_url(&ctx.http, Permissions::empty()).await {
            Ok(url) => {
                println!("You can invite me using this url! {}", &url);
            }
            Err(why) => {
                eprintln!("Error getting invite url: {:?}", why);
            }
        };
    }
}

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument.\n Example: !help ctf list\n"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[strikethrough_commands_tip_in_guild = ""]
#[strikethrough_commands_tip_in_dm = ""]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("{}: {}", msg.author.name, msg.content);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Load all environment variables from .env file.
    dotenv().ok();

    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in your environment (DISCORD_TOKEN)");
    let owner_id_str = env::var("OWNER_ID").expect("Expected an OWNER_ID in your environment!");

    let owner_id = owner_id_str
        .parse::<u64>()
        .expect("Unable to parse OWNER_ID into u64... Did you put it in correctly?");

    let mut owners = HashSet::new();
    owners.insert(UserId(owner_id));

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefixes(vec!["!", ".", "~"])
                .owners(owners)
                .with_whitespace(true)
        })
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&HELP)
        .group(&HTBER_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Copy the token so we can use it for HTB as well
    let token_copy = token.clone();

    // Load all required values for HTB API
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

    let htb_config = HTBAPIConfig {
        email,
        password: pass,
        team_id,
    };

    let htb_api = new_htbapi_instance(htb_config)
        .await
        .expect("Error when creating HTBApi instance...");
    let http = Http::new_with_token(&token_copy);
    let channel_id = ChannelId(htb_channel_id);

    let scheduler_data = ScheduleRunnerData {
        htb_api,
        http,
        channel_id,
    };

    let threadsafe_data = Arc::new(Mutex::new(scheduler_data));

    // Load the categories into the cache once on boot.
    let data = threadsafe_data.lock().unwrap();
    if let Err(why) = load_categories_to_cache(&data.htb_api).await {
        eprintln!("Error loading categories to cache... {}", why);
    }
    std::mem::drop(data);

    let data_arc1 = threadsafe_data.clone();
    let data_arc2 = threadsafe_data.clone();

    std::thread::spawn(move || loop {
        if let Ok(lock) = data_arc1.lock().as_mut() {
            println!("Updating HTB challenges...");
            if let Err(why) = update_htb_challenges(&lock.htb_api) {
                eprintln!("Error updating HTB challenges/machines: {:?}", why);
            }

            println!("Processing current rank...");
            if let Err(why) = process_rank_status(lock) {
                eprintln!("Error updating team rank status: {:?}", why);
            }
        }

        // Sleep for a day.
        std::thread::sleep(Duration::from_secs(86400));
    });

    std::thread::spawn(move || loop {
        if let Ok(lock) = data_arc2.lock().as_mut() {
            println!("Polling for new solves...");
            match process_new_solves(lock) {
                Ok(_) => {
                    println!("Successfully processed HTB solves!");
                }
                Err(why) => {
                    eprintln!("Error processing HTB solves: {:?}", why);
                }
            }
        }

        // Sleep for 5 minutes.
        std::thread::sleep(Duration::from_secs(60 * 5));
    });

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
