use crate::error::on_error;
use crate::event::event_handler;
use crate::schedule::DataJob;
use crate::schedule::StoredJob;
use crate::schedule::create_scheduled_day_post;
use ::serenity::all::ChannelId;
use poise::serenity_prelude::ClientBuilder;
use poise::serenity_prelude::GatewayIntents;
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::sync::Mutex;

use ::serenity::all::CreateInteractionResponseFollowup;
use std::env;
use tokio_cron_scheduler::JobScheduler;

pub(crate) mod commands;
pub(crate) mod error;
pub(crate) mod list;
pub(crate) mod schedule;
pub(crate) mod types;
pub(crate) mod event;

pub struct Data {
    sched: Arc<Mutex<JobScheduler>>,
    job_uuids: Arc<Mutex<Vec<DataJob>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");

    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::builtin::register(),
            commands::list::daily_menu(),
            commands::list::weekly_menu(),
            commands::schedule::schedule_day(),
            commands::schedule::list_scheduled(),
            commands::schedule::delete_scheduled(),
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
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                ctx.online();

                let mut uuids: Vec<DataJob> = Vec::new();

                let jobs: Vec<StoredJob> =
                    serde_json::from_str(&read_to_string("jobs.json").unwrap_or("[]".to_string()))?;

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
