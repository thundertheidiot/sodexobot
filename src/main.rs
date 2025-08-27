use poise::serenity_prelude::ClientBuilder;
use poise::serenity_prelude::CreateInteractionResponseMessage;
use poise::serenity_prelude::GatewayIntents;

use ::serenity::all::CreateInteractionResponse;
use ::serenity::all::CreateInteractionResponseFollowup;
use ::serenity::all::Interaction;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use std::env;

mod lista;

use crate::lista::extra_info::extra_info;
use crate::lista::ruokalista;

pub struct Data {
    member: i8,
}

struct UserData {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Component(c) = interaction {
                let id = &c.data.custom_id;

                if id.starts_with("infoday") {
                    extra_info(ctx, c).await?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");

    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![ruokalista()],
        on_error: |error| Box::pin(on_error(error)),
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { member: 1 })
            })
        })
        .options(options)
        .build();

    let intents = GatewayIntents::non_privileged();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
