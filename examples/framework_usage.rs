use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var, sync::Mutex, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type PrefixContext<'a> = poise::PrefixContext<'a, Data, Error>;

struct Data {
    votes: Mutex<HashMap<String, u32>>,
    owner_id: serenity::UserId,
}

/// Vote for something
///
/// Enter `~vote pumpkin` to vote for pumpkins
#[poise::command(slash_command)]
async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!(
        "Successfully voted for {0}. {0} now has {1} votes!",
        choice, num_votes
    );
    poise::say_reply(ctx, response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// ~getvotes
/// ~getvotes pumpkin
/// ```
#[poise::command(slash_command, track_edits, aliases("votes"))]
async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        poise::say_reply(ctx, response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
            response += &format!("{}: {} votes\n", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        poise::say_reply(ctx, response).await?;
    };

    Ok(())
}

/// Add two numbers
#[poise::command(slash_command, track_edits)]
async fn add(
    ctx: Context<'_>,
    #[description = "First operand"] a: f64,
    #[description = "Second operand"] b: f32,
) -> Result<(), Error> {
    poise::say_reply(ctx, format!("Result: {}", a + b as f64)).await?;

    Ok(())
}

#[derive(Debug, poise::SlashChoiceParameter)]
enum MyStringChoice {
    #[name = "The first choice"]
    ChoiceA,
    #[name = "The second choice"]
    ChoiceB,
}

/// Dummy command to test slash command choice parameters
#[poise::command(slash_command)]
async fn choice(
    ctx: Context<'_>,
    #[description = "The choice you want to choose"] choice: poise::Wrapper<MyStringChoice>,
) -> Result<(), Error> {
    let choice = choice.0;

    poise::say_reply(ctx, format!("You entered {:?}", choice)).await?;
    Ok(())
}

/// Show this help menu
#[poise::command(track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    poise::defaults::help(
        ctx,
        command.as_deref(),
        "This is an example bot made to showcase features of my custom Discord bot framework",
        poise::defaults::HelpResponseMode::Ephemeral,
    )
    .await?;
    Ok(())
}

async fn is_owner(ctx: crate::PrefixContext<'_>) -> Result<bool, Error> {
    Ok(ctx.msg.author.id == ctx.data.owner_id)
}

/// Register slash commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(check = "is_owner", hide_in_help)]
async fn register(ctx: PrefixContext<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::defaults::register_slash_commands(ctx, global).await?;

    Ok(())
}

async fn on_error(error: Error, ctx: poise::ErrorContext<'_, Data, Error>) {
    match ctx {
        poise::ErrorContext::Setup => panic!("Failed to start bot: {:?}", error),
        poise::ErrorContext::Command(ctx) => {
            println!("Error in command `{}`: {:?}", ctx.command().name(), error)
        }
        _ => println!("Other error: {:?}", error),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut options = poise::FrameworkOptions {
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        on_error: |error, ctx| Box::pin(on_error(error, ctx)),
        ..Default::default()
    };

    options.command(vote(), |f| f);
    options.command(getvotes(), |f| f);
    options.command(help(), |f| f);
    options.command(register(), |f| f);
    options.command(add(), |f| f);
    options.command(choice(), |f| f);

    let framework = poise::Framework::new(
        "~".to_owned(), // prefix
        serenity::ApplicationId(var("APPLICATION_ID")?.parse()?),
        move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                    owner_id: serenity::UserId(var("OWNER_ID")?.parse()?),
                })
            })
        },
        options,
    );
    framework
        .start(serenity::ClientBuilder::new(&var("TOKEN")?))
        .await?;

    Ok(())
}
