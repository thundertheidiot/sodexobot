use crate::lista::viikon_lista;
use crate::schedule::delete_scheduled;
use crate::schedule::list_scheduled;
use crate::schedule::DataJob;
use crate::schedule::StoredJob;
use crate::schedule::create_scheduled_day_post;
use crate::schedule::schedule_day;
use ::serenity::all::ChannelId;
use ::serenity::all::CreateMessage;
use poise::serenity_prelude::ClientBuilder;
use poise::serenity_prelude::CreateInteractionResponseMessage;
use poise::serenity_prelude::GatewayIntents;
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use ::serenity::all::CreateInteractionResponse;
use ::serenity::all::CreateInteractionResponseFollowup;
use ::serenity::all::Interaction;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use std::env;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

pub(crate) mod lista;
pub(crate) mod schedule;
pub(crate) mod types;

use crate::lista::extra_info::extra_info;
use crate::lista::ruokalista;

pub struct Data {
    sched: Arc<Mutex<JobScheduler>>,
    job_uuids: Arc<Mutex<Vec<DataJob>>>,
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

#[poise::command(prefix_command, hide_in_help = true)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");

    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![
	    register(),
	    ruokalista(),
	    viikon_lista(),
	    schedule_day(),
	    list_scheduled(),
	    delete_scheduled(),
	],
        on_error: |error| Box::pin(on_error(error)),
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let scheduler = JobScheduler::new().await?;

    scheduler.start().await?;

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

		ctx.online();

                let mut uuids: Vec<DataJob> = Vec::new();

		let jobs: Vec<StoredJob> = serde_json::from_str(&read_to_string("jobs.json").unwrap_or("[]".to_string()))?;

                for i in jobs {
                    if let Ok(job) =
                        create_scheduled_day_post(ctx, &i.cron, ChannelId::new(i.channel_id))
                    {
                        uuids.push(DataJob {
                            uuid: job.guid(),
                            cron: i.cron,
                            channel_id: i.channel_id,
                        });

                        scheduler.add(job).await?;
                    }
                }

                Ok(Data {
                    sched: Arc::new(Mutex::new(scheduler)),
                    job_uuids: Arc::new(Mutex::new(uuids)),
                })
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
