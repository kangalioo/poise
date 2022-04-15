/**
Poise supports several pre-command checks (sorted by order of execution):
- owners_only
- required_permissions
- required_bot_permissions
- global check function
- command-specific check function
- cooldowns
*/
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}

/// A moderator-only command, using required_permissions
#[poise::command(
    prefix_command,
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn modonly(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You are a mod because you were able to invoke this command")
        .await?;
    Ok(())
}

/// Deletes the given message
#[poise::command(
    prefix_command,
    slash_command,
    required_bot_permissions = "MANAGE_MESSAGES"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Message to be deleted"] msg: serenity::Message,
) -> Result<(), Error> {
    msg.delete(ctx.discord()).await?;
    Ok(())
}

/// Returns true if username is Ferris
async fn is_ferris(ctx: Context<'_>) -> Result<bool, Error> {
    let nickname = match ctx.guild_id() {
        Some(guild_id) => ctx.author().nick_in(ctx.discord(), guild_id).await,
        None => None,
    };
    let name = nickname.as_ref().unwrap_or(&ctx.author().name);

    Ok(name.eq_ignore_ascii_case("ferris"))
}

/// Crab party... only for "Ferris"!
#[poise::command(prefix_command, slash_command, check = "is_ferris")]
pub async fn ferrisparty(ctx: Context<'_>) -> Result<(), Error> {
    let response = "```\n".to_owned()
        + &r"    _~^~^~_
\) /  o o  \ (/
  '_   ¬   _'
  | '-----' |
"
        .repeat(3)
        + "```";
    ctx.say(response).await?;
    Ok(())
}

/// Add two numbers
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    // All cooldowns in seconds
    global_cooldown = 1,
    user_cooldown = 5,
    guild_cooldown = 2,
    channel_cooldown = 2,
    member_cooldown = 3,
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "First operand"] a: f64,
    #[description = "Second operand"]
    #[min = -15]
    #[max = 28.765]
    b: f32,
) -> Result<(), Error> {
    ctx.say(format!("Result: {}", a + b as f64)).await?;

    Ok(())
}

/// Get the guild name (guild-only)
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn get_guild_name(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "The name of this guild is: {}",
        ctx.guild().unwrap().name
    ))
    .await?;

    Ok(())
}

/// A dm-only command
#[poise::command(prefix_command, slash_command, dm_only)]
pub async fn only_in_dms(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is a dm channel").await?;

    Ok(())
}

/// Only runs on NSFW channels
#[poise::command(prefix_command, slash_command, nsfw_only)]
pub async fn lennyface(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("( ͡° ͜ʖ ͡°)").await?;

    Ok(())
}
