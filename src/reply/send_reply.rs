//! All functions to actually send a reply

use crate::serenity_prelude as serenity;

/// Send a message in the given context: normal message if prefix command, interaction response
/// if application command.
///
/// If you just want to send a string, use [`say_reply`].
///
/// Note: panics when called in an autocomplete context!
///
/// ```rust,no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ctx: poise::Context<'_, (), ()> = todo!();
/// ctx.send(|f| f
///     .content("Works for slash and prefix commands")
///     .embed(|f| f
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<'att, U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => {
            crate::ReplyHandle::Known(crate::send_prefix_reply(ctx, builder).await?)
        }
        crate::Context::Application(ctx) => crate::send_application_reply(ctx, builder).await?,
    })
}

/// Shorthand of [`send_reply`] for text-only messages
///
/// Note: panics when called in an autocomplete context!
pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into())).await
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::ApplicationCommandInteraction::create_followup_message) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<'att, U, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    let mut data = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        allowed_mentions: ctx.framework.options().allowed_mentions.clone(),
        ..Default::default()
    };
    builder(&mut data);
    _send_application_reply(ctx, data).await
}

/// private version of [`send_application_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_application_reply<'a, 'b, U, E>(
    ctx: crate::ApplicationContext<'b, U, E>,
    mut data: crate::CreateReply<'a>,
) -> Result<crate::ReplyHandle<'b>, serenity::Error> {
    let interaction = match ctx.interaction {
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
            return Ok(crate::ReplyHandle::Autocomplete)
        }
    };

    if let Some(callback) = ctx.framework.options().reply_callback {
        callback(ctx.into(), &mut data);
    }

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    Ok(if has_sent_initial_response {
        crate::ReplyHandle::Known(Box::new(
            interaction
                .create_followup_message(ctx.discord, |f| {
                    data.to_slash_followup_response(f);
                    f
                })
                .await?,
        ))
    } else {
        interaction
            .create_interaction_response(ctx.discord, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|f| {
                        data.to_slash_initial_response(f);
                        f
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        crate::ReplyHandle::Unknown {
            http: &ctx.discord.http,
            interaction,
        }
    })
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'att, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    let mut reply = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        allowed_mentions: ctx.framework.options().allowed_mentions.clone(),
        ..Default::default()
    };
    builder(&mut reply);
    _send_prefix_reply(ctx, reply).await
}

/// private version of [`send_prefix_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_prefix_reply<'a, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    mut reply: crate::CreateReply<'a>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    if let Some(callback) = ctx.framework.options().reply_callback {
        callback(ctx.into(), &mut reply);
    }

    // This must only return None when we _actually_ want to reuse the existing response! There are
    // no checks later
    let lock_edit_tracker = || {
        if ctx.command.reuse_response {
            if let Some(edit_tracker) = &ctx.framework.options().prefix_options.edit_tracker {
                return Some(edit_tracker.write().unwrap());
            }
        }
        None
    };

    let existing_response = lock_edit_tracker()
        .as_mut()
        .and_then(|t| t.find_bot_response(ctx.msg.id))
        .cloned();

    Ok(Box::new(if let Some(mut response) = existing_response {
        response
            .edit(ctx.discord, |f| {
                // Reset the message. We don't want leftovers of the previous message (e.g. user
                // sends a message with `.content("abc")` in a track_edits command, and the edited
                // message happens to contain embeds, we don't want to keep those embeds)
                // (*f = Default::default() won't do)
                f.content("");
                f.set_embeds(Vec::new());
                f.components(|b| b);
                // TODO: The new builder doesn't support this. Needs `set_attachments` method.
                // f.0.insert("attachments", serenity::json::json! { [] });

                reply.to_prefix_edit(f);
                f
            })
            .await?;

        // If the entry still exists after the await, update it to the new contents
        if let Some(mut edit_tracker) = lock_edit_tracker() {
            edit_tracker.set_bot_response(ctx.msg, response.clone());
        }

        response
    } else {
        let new_response = ctx
            .msg
            .channel_id
            .send_message(ctx.discord, |m| {
                reply.to_prefix(m);
                m
            })
            .await?;
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.set_bot_response(ctx.msg, new_response.clone());
        }

        new_response
    }))
}
