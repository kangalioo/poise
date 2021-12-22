use super::Invocation;
use syn::spanned::Spanned as _;

// ngl this is ugly
// transforms a type of form `OuterType<T>` into `T`
fn extract_type_parameter<'a>(outer_type: &str, t: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(path) = t {
        if path.path.segments.len() == 1 {
            let path = &path.path.segments[0];
            if path.ident == outer_type {
                if let syn::PathArguments::AngleBracketed(generics) = &path.arguments {
                    if generics.args.len() == 1 {
                        if let syn::GenericArgument::Type(t) = &generics.args[0] {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn generate_slash_parameters(
    inv: &Invocation,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut parameter_structs = Vec::new();
    for param in &inv.parameters {
        let description = param.more.description.as_ref().ok_or_else(|| {
            syn::Error::new(
                param.span,
                "slash command parameters must have a description",
            )
        })?;

        let (mut required, type_) = match extract_type_parameter("Option", &param.type_)
            .or_else(|| extract_type_parameter("Vec", &param.type_))
        {
            Some(t) => (false, t),
            None => (true, &param.type_),
        };

        // Don't require user to input a value for flags - use false as default value (see below)
        if param.more.flag {
            required = false;
        }

        let param_name = &param.name;
        let autocomplete_callback = match &param.more.autocomplete {
            Some(autocomplete_fn) => {
                quote::quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    interaction: &poise::serenity_prelude::AutocompleteInteraction,
                    options: &[poise::serenity_prelude::ApplicationCommandInteractionDataOption],
                | Box::pin(async move {
                    use ::poise::futures::{Stream, StreamExt};

                    let choice = match options
                        .iter()
                        .find(|option| option.focused && option.name == stringify!(#param_name))
                    {
                        Some(x) => x,
                        None => return Ok(()),
                    };

                    let json_value = choice.value
                        .as_ref()
                        .ok_or(::poise::SlashArgError::CommandStructureMismatch("expected argument value"))?;
                    let partial_input = (&&&&&std::marker::PhantomData::<#type_>).extract_partial(json_value)?;

                    let choices_stream = ::poise::into_stream!(
                        #autocomplete_fn(ctx.into(), partial_input).await
                    );
                    let choices_json = choices_stream
                        .take(25)
                        .map(|value| poise::AutocompleteChoice::from(value))
                        .map(|choice| poise::serenity::json::json!({
                            "name": choice.name,
                            "value": (&&&&&std::marker::PhantomData::<#type_>).into_json(choice.value),
                        }))
                        .collect()
                        .await;
                    let choices_json = poise::serenity::json::Value::Array(choices_json);

                    if let Err(e) = interaction
                        .create_autocomplete_response(
                            &ctx.discord.http,
                            |b| b.set_choices(choices_json),
                        )
                        .await
                    {
                        println!("Warning: couldn't send autocomplete response: {}", e);
                    }

                    Ok(())
                })) }
            }
            None => quote::quote! { None },
        };

        let is_autocomplete = param.more.autocomplete.is_some();
        parameter_structs.push((
            quote::quote! {
                ::poise::SlashCommandParameter {
                    builder: |o| (&&&&&std::marker::PhantomData::<#type_>).create(o)
                        .required(#required)
                        .name(stringify!(#param_name))
                        .description(#description)
                        .set_autocomplete(#is_autocomplete),
                    autocomplete_callback: #autocomplete_callback,
                }
            },
            required,
        ));
    }
    // Sort the parameters so that optional parameters come last - Discord requires this order
    parameter_structs.sort_by_key(|(_, required)| !required);
    Ok(parameter_structs
        .into_iter()
        .map(|(builder, _)| builder)
        .collect::<Vec<_>>())
}

pub fn generate_slash_action(inv: &Invocation) -> proc_macro2::TokenStream {
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv
        .parameters
        .iter()
        .map(|p| match p.more.flag {
            true => syn::parse_quote! { FLAG },
            false => p.type_.clone(),
        })
        .collect::<Vec<_>>();

    quote::quote! {
        |ctx, args| Box::pin(async move {
            // idk why this can't be put in the macro itself (where the lint is triggered) and
            // why clippy doesn't turn off this lint inside macros in the first place
            #[allow(clippy::needless_question_mark)]

            let ( #( #param_names, )* ) = ::poise::parse_slash_args!(
                ctx.discord, ctx.interaction.guild_id(), ctx.interaction.channel_id(), args =>
                #( (#param_names: #param_types), )*
            ).await.map_err(|error| match error {
                poise::SlashArgError::CommandStructureMismatch(error) => {
                    poise::FrameworkError::CommandStructureMismatch { ctx, error }
                },
                poise::SlashArgError::Parse(error) => {
                    poise::FrameworkError::ArgumentParse { ctx: ctx.into(), error }
                },
            })?;

            inner(ctx.into(), #( #param_names, )*)
                .await
                .map_err(|error| poise::FrameworkError::Command {
                    error,
                    ctx: ctx.into(),
                })
        })
    }
}

pub fn generate_context_menu_action(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_type = match &*inv.parameters {
        [single_param] => &single_param.type_,
        _ => {
            return Err(syn::Error::new(
                inv.function.sig.inputs.span(),
                "Context menu commands require exactly one parameter",
            ))
        }
    };

    Ok(quote::quote! {
        <#param_type as ::poise::ContextMenuParameter<_, _>>::to_action(|ctx, value| {
            Box::pin(async move {
                inner(ctx.into(), value)
                    .await
                    .map_err(|error| poise::FrameworkError::Command {
                        error,
                        ctx: ctx.into(),
                    })
            })
        })
    })
}