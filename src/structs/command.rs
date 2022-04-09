//! The Command struct, which stores all information about a single framework command

use crate::{serenity_prelude as serenity, BoxFuture};

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
#[derive(Default)]
pub struct Command<U, E> {
    // =============
    /// Callback to execute when this command is invoked in a prefix context
    pub prefix_action: Option<
        for<'a> fn(
            crate::PrefixContext<'a, U, E>,
        ) -> BoxFuture<'a, Result<(), crate::FrameworkError<'a, U, E>>>,
    >,
    /// Callback to execute when this command is invoked in a slash context
    pub slash_action: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, U, E>,
        ) -> BoxFuture<'a, Result<(), crate::FrameworkError<'a, U, E>>>,
    >,
    /// Callback to execute when this command is invoked in a context menu context
    ///
    /// The enum variant shows which Discord item this context menu command works on
    pub context_menu_action: Option<crate::ContextMenuCommandAction<U, E>>,

    // ============= Command type agnostic data
    /// Subcommands of this command, if any
    pub subcommands: Vec<Command<U, E>>,
    /// Main name of the command. Aliases (prefix-only) can be set in [`Self::aliases`].
    pub name: &'static str,
    /// Full name including parent command names.
    ///
    /// Initially set to just [`Self::name`] and properly populated when the framework is started.
    pub qualified_name: String,
    /// A string to identify this particular command within a list of commands.
    ///
    /// Can be configured via the [`crate::command`] macro (though it's probably not needed for most
    /// bots). If not explicitly configured, it falls back to prefix command name, slash command
    /// name, or context menu command name (in that order).
    pub identifying_name: String,
    /// Identifier for the category that this command will be displayed in for help commands.
    pub category: Option<&'static str>,
    /// Whether to hide this command in help menus.
    pub hide_in_help: bool,
    /// Short description of the command. Displayed inline in help menus and similar.
    pub inline_help: Option<&'static str>,
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub multiline_help: Option<fn() -> String>,
    /// Handles command cooldowns. Mainly for framework internal use
    pub cooldowns: std::sync::Mutex<crate::Cooldowns>,
    /// After the first response, whether to post subsequent responses as edits to the initial
    /// message
    ///
    /// Note: in prefix commands, this only has an effect if
    /// `crate::PrefixFrameworkOptions::edit_tracker` is set.
    pub reuse_response: bool,
    /// Permissions which users must have to invoke this command.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_permissions: serenity::Permissions,
    /// Permissions without which command execution will fail. You can set this to fail early and
    /// give a descriptive error message in case the
    /// bot hasn't been assigned the minimum permissions by the guild admin.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_bot_permissions: serenity::Permissions,
    /// If true, only users from the [owners list](crate::FrameworkOptions::owners) may use this
    /// command.
    pub owners_only: bool,
    /// If true, only people in guilds may use this command
    pub guild_only: bool,
    /// If true, the command may only run in DMs
    pub dm_only: bool,
    /// If true, the command may only run in NSFW channels
    pub nsfw_only: bool,
    /// Command-specific override for [`crate::FrameworkOptions::on_error`]
    pub on_error: Option<fn(crate::FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(crate::Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// List of parameters for this command
    ///
    /// Used for registering and parsing slash commands. Can also be used in help commands
    pub parameters: Vec<crate::CommandParameter<U, E>>,

    // ============= Prefix-specific data
    /// Alternative triggers for the command (prefix-only)
    pub aliases: &'static [&'static str],
    /// Whether to rerun the command if an existing invocation message is edited (prefix-only)
    pub invoke_on_edit: bool,
    /// Whether to broadcast a typing indicator while executing this commmand (prefix-only)
    pub broadcast_typing: bool,

    // ============= Application-specific data
    /// Context menu specific name for this command, displayed in Discord's context menu
    pub context_menu_name: Option<&'static str>,
    /// Whether responses to this command should be ephemeral by default (application-only)
    pub ephemeral: bool,
}

impl<U, E> PartialEq for Command<U, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl<U, E> Eq for Command<U, E> {}

impl<U, E> std::fmt::Debug for Command<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            prefix_action,
            slash_action,
            context_menu_action,
            subcommands,
            name,
            qualified_name,
            identifying_name,
            category,
            hide_in_help,
            inline_help,
            multiline_help,
            cooldowns,
            required_permissions,
            required_bot_permissions,
            owners_only,
            guild_only,
            dm_only,
            nsfw_only,
            on_error,
            check,
            parameters,
            aliases,
            invoke_on_edit,
            reuse_response,
            broadcast_typing,
            context_menu_name,
            ephemeral,
        } = self;

        f.debug_struct("Command")
            .field("prefix_action", &prefix_action.map(|f| f as *const ()))
            .field("slash_action", &slash_action.map(|f| f as *const ()))
            .field("context_menu_action", context_menu_action)
            .field("subcommands", subcommands)
            .field("name", name)
            .field("qualified_name", qualified_name)
            .field("identifying_name", identifying_name)
            .field("category", category)
            .field("hide_in_help", hide_in_help)
            .field("inline_help", inline_help)
            .field("multiline_help", multiline_help)
            .field("cooldowns", cooldowns)
            .field("required_permissions", required_permissions)
            .field("required_bot_permissions", required_bot_permissions)
            .field("owners_only", owners_only)
            .field("guild_only", guild_only)
            .field("dm_only", dm_only)
            .field("nsfw_only", nsfw_only)
            .field("on_error", &on_error.map(|f| f as *const ()))
            .field("check", &check.map(|f| f as *const ()))
            .field("parameters", parameters)
            .field("aliases", aliases)
            .field("invoke_on_edit", invoke_on_edit)
            .field("reuse_response", reuse_response)
            .field("broadcast_typing", broadcast_typing)
            .field("context_menu_name", context_menu_name)
            .field("ephemeral", ephemeral)
            .finish()
    }
}

impl<U, E> Command<U, E> {
    /// Serializes this Command into an application command option, which is the form which Discord
    /// requires subcommands to be in
    fn create_as_subcommand(&self) -> Option<serenity::CreateApplicationCommandOption> {
        self.slash_action?;

        let mut builder = serenity::CreateApplicationCommandOption::default();
        builder
            .name(self.name)
            .description(self.inline_help.unwrap_or("A slash command"));

        if self.subcommands.is_empty() {
            builder.kind(serenity::ApplicationCommandOptionType::SubCommand);

            for param in &self.parameters {
                // Using `?` because if this command has slash-incompatible parameters, we cannot
                // just ignore them but have to abort the creation process entirely
                builder.add_sub_option(param.create_as_slash_command_option()?);
            }
        } else {
            builder.kind(serenity::ApplicationCommandOptionType::SubCommandGroup);

            for subcommand in &self.subcommands {
                if let Some(subcommand) = subcommand.create_as_subcommand() {
                    builder.add_sub_option(subcommand);
                }
            }
        }

        Some(builder)
    }

    /// Generates a slash command builder from this [`Command`] instance. This can be used
    /// to register this command on Discord's servers
    pub fn create_as_slash_command(&self) -> Option<serenity::CreateApplicationCommand> {
        self.slash_action?;

        let mut builder = serenity::CreateApplicationCommand::default();
        builder
            .name(self.name)
            .description(self.inline_help.unwrap_or("A slash command"));

        if self.subcommands.is_empty() {
            for param in &self.parameters {
                // Using `?` because if this command has slash-incompatible parameters, we cannot
                // just ignore them but have to abort the creation process entirely
                builder.add_option(param.create_as_slash_command_option()?);
            }
        } else {
            for subcommand in &self.subcommands {
                if let Some(subcommand) = subcommand.create_as_subcommand() {
                    builder.add_option(subcommand);
                }
            }
        }

        Some(builder)
    }

    /// Generates a context menu command builder from this [`Command`] instance. This can be used
    /// to register this command on Discord's servers
    pub fn create_as_context_menu_command(&self) -> Option<serenity::CreateApplicationCommand> {
        let context_menu_action = self.context_menu_action?;

        let mut builder = serenity::CreateApplicationCommand::default();
        builder
            .name(self.context_menu_name.unwrap_or(self.name))
            .kind(match context_menu_action {
                crate::ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                crate::ContextMenuCommandAction::Message(_) => {
                    serenity::ApplicationCommandType::Message
                }
            });

        Some(builder)
    }

    /// **Deprecated**
    #[deprecated = "Please use `crate::Command { category: \"...\", ..command() }` instead"]
    pub fn category(&mut self, category: &'static str) -> &mut Self {
        self.category = Some(category);
        self
    }

    /// Insert a subcommand
    pub fn subcommand(
        &mut self,
        mut subcommand: crate::Command<U, E>,
        meta_builder: impl FnOnce(&mut Self) -> &mut Self,
    ) -> &mut Self {
        meta_builder(&mut subcommand);
        self.subcommands.push(subcommand);
        self
    }
}
