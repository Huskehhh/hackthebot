use hackthebot::{get_challenge_category_from_id, graphql::graphql_types::Challenge};
use serenity::framework::standard::CommandResult;
use serenity::{builder::CreateEmbed, http::Http, model::id::ChannelId};

#[derive(Debug)]
pub struct SolveToAnnounce<'a> {
    pub solver: String,
    pub user_id: i64,
    pub solve_type: String,
    pub challenge: &'a Challenge,
}

pub fn populate_embed_from_challenge(
    challenge: &Challenge,
    e: &mut CreateEmbed,
    solving_users: Option<Vec<String>>,
) {
    let challenge_category_name = get_challenge_category_from_id(challenge.category);

    e.title(format!("🏴 {}", challenge.name));
    e.field("📚 Category", &challenge_category_name, true);
    e.field("💰 Points", challenge.points, true);

    if let Some(solving_users) = solving_users {
        let solving_string = solving_users.join(", ");
        e.field("🏴‍ Solved", solving_string, true);
    }

    if let Some(avatar) = &challenge.machine_avatar {
        e.thumbnail(format!("https://www.hackthebox.eu/{}", avatar));
    }
}

fn capitalise_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

pub async fn announce_solve(
    solve: &SolveToAnnounce<'_>,
    channel_id: &ChannelId,
    http: &Http,
) -> CommandResult {
    let challenge = &solve.challenge;
    let challenge_category_name = get_challenge_category_from_id(challenge.category);

    let solve_type = capitalise_first(&solve.solve_type);

    // Build up the content based around the solve type
    let content = if solve_type.eq("Challenge") {
        format!(
            "🏴 {} has been solved by {}",
            &challenge.name, &solve.solver
        )
    } else {
        format!(
            "🏴 {} has been owned by {} on {}",
            solve_type, &solve.solver, &challenge.name
        )
    };

    channel_id
        .send_message(http, |message| {
            message.embed(|e| {
                e.title(content);
                e.field("📚 Category", &challenge_category_name, true);
                e.field("💰 Points", &challenge.points, true);

                if let Some(avatar) = &solve.challenge.machine_avatar {
                    e.thumbnail(format!("https://www.hackthebox.eu/{}", avatar));
                }

                e
            })
        })
        .await?;

    Ok(())
}
