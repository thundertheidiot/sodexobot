use crate::Data;
use crate::Error;
use poise::CreateReply;
use poise::FrameworkError;
use poise::FrameworkError::EventHandler;
use poise::FrameworkError::{Command, Setup};

pub async fn on_error(error: FrameworkError<'_, Data, Error>) {
    match error {
        Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        Command { error, ctx, .. } => {
            println!("Error in command `{}`: {error:?}", ctx.command().name);

            if let Err(e) = ctx
                .send(
                    CreateReply::default()
                        .content(format!("Error: `{error:?}`"))
                        .ephemeral(true),
                )
                .await
            {
                println!("unable to send error message: {e}");
            }
        }
        EventHandler { error, event, .. } => {
            println!("Error in event `{}`: {error:?}", event.snake_case_name());
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}
