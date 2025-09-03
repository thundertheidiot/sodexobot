pub mod schedule;
pub mod list;

pub mod builtin {
    use crate::{Context, Error};

    #[poise::command(prefix_command, hide_in_help = true)]
    pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
        poise::builtins::register_application_commands_buttons(ctx).await?;
        Ok(())
    }
}
