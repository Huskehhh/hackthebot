use hackthebot::graphql::graphql_provider::GraphQLProvider;
use hackthebot::{get_hasura_client, HASURA_PROVIDER};
use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};

use serenity::model::channel::Message;

use crate::discord_utils::populate_embed_from_challenge;

#[group]
#[commands(search, solves)]
#[prefixes("htb", "h")]
pub struct HTBer;

#[command]
#[allowed_roles("CTFer")]
#[example("\"Challenge name\"")]
#[description = "Searches for the status of the given challenge"]
async fn search(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 1 {
        let challenge_name = args.single_quoted::<String>()?;

        let hasura_provider = HASURA_PROVIDER.get_or_init(get_hasura_client).await;

        let challenges = hasura_provider
            .get_challenges_for_name(&challenge_name)
            .await?;

        if !challenges.is_empty() {
            for challenge in challenges {
                let solving_users_vec = hasura_provider
                    .get_users_who_solved_challenge(challenge.htb_id)
                    .await?;
                let solving_users = if !solving_users_vec.is_empty() {
                    Some(solving_users_vec)
                } else {
                    None
                };

                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            populate_embed_from_challenge(&challenge, e, solving_users);
                            e
                        })
                    })
                    .await?;
            }
        } else {
            msg.reply(&ctx.http, "No challenge found by that name!")
                .await?;
        }
    } else {
        msg.reply(&ctx.http, "Usage: ``!htb search \"Challenge name\"``")
            .await?;
    }

    Ok(())
}

#[command]
#[allowed_roles("CTFer")]
#[example("\"Username\"")]
#[description = "Searches for solves of a given user"]
async fn solves(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 1 {
        let username = args.single_quoted::<String>()?;
        let hasura_provider = HASURA_PROVIDER.get_or_init(get_hasura_client).await;
        let solves = hasura_provider
            .get_solves_for_user_by_name(username)
            .await?;

        if !solves.is_empty() {
            for solve in solves {
                let challenge = hasura_provider
                    .get_challenge_by_id(solve.challenge_id)
                    .await?;

                if let Some(first_challenge) = challenge.first() {
                    let solving_users_vec = hasura_provider
                        .get_users_who_solved_challenge(first_challenge.htb_id)
                        .await?;
                    let solving_users = if !solving_users_vec.is_empty() {
                        Some(solving_users_vec)
                    } else {
                        None
                    };

                    msg.channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                populate_embed_from_challenge(first_challenge, e, solving_users);
                                e
                            })
                        })
                        .await?;
                }
            }
        } else {
            msg.reply(&ctx.http, "No solves found for that user!")
                .await?;
        }
    } else {
        msg.reply(&ctx.http, "Usage: ``!htb solves \"Username\"``")
            .await?;
    }

    Ok(())
}
