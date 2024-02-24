use color_eyre::eyre::Result;
use hackthebot::Challenge;
use serenity::{http::Http, model::id::ChannelId};

#[derive(Debug)]
pub struct SolveToAnnounce {
    pub solver: String,
    pub user_id: i64,
    pub solve_type: String,
    pub challenge: Challenge,
}

pub fn get_challenge_category(challenge: &Challenge) -> String {
    if challenge.challenge_type.to_lowercase().contains("machine") {
        "Machine".to_owned()
    } else {
        match &challenge.challenge_category {
            Some(challenge_cat) => challenge_cat.clone(),
            _ => challenge.challenge_type.clone(),
        }
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
    solve: &SolveToAnnounce,
    channel_id: &ChannelId,
    http: &Http,
) -> Result<()> {
    let challenge = &solve.challenge;
    let category = get_challenge_category(challenge);

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
                e.field("📚 Category", &category, true);
                e.field("💰 Points", challenge.points, true);

                if let Some(avatar) = &solve.challenge.machine_avatar {
                    e.thumbnail(format!("https://www.hackthebox.eu/{avatar}"));
                }

                e
            })
        })
        .await?;

    Ok(())
}
