use serde::Deserialize;
use crate::lista::fetch_day;
use crate::lista::fmt_day;
use crate::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude::CreateMessage;
use serde::Serialize;
use serenity::all::ChannelId;
use serenity::all::MessageReference;
use serenity::all::MessageReferenceKind;
use std::fs::write;
use std::sync::Arc;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobSchedulerError;
use tokio_cron_scheduler::job::JobLocked;
use uuid::Uuid;

pub struct DataJob {
    pub uuid: Uuid,
    pub cron: Box<str>,
    pub channel_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredJob {
    pub cron: Box<str>,
    pub channel_id: u64,
}

impl Into<StoredJob> for &DataJob {
    fn into(self) -> StoredJob {
        StoredJob {
            cron: self.cron.clone(),
            channel_id: self.channel_id,
        }
    }
}

pub fn create_scheduled_day_post<S: ToString>(
    ctx: &poise::serenity_prelude::Context,
    cron: S,
    channel_id: ChannelId,
) -> Result<JobLocked, JobSchedulerError> {
    let ctx = Arc::new(ctx.clone());

    Job::new_async(cron, move |_uuid, _l| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let day = chrono::Local::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string();

            match fetch_day(&day).await {
                Ok(menu) => {
                    let reply = fmt_day(&day, menu, None);

                    let m = reply.to_prefix(MessageReference::new(
                        MessageReferenceKind::Default,
                        channel_id,
                    ));
                    if let Err(e) = channel_id.send_message(&ctx.http, m).await {
                        println!("Error {e:#?}");
                    }
                }
                Err(e) => {
                    if let Err(e) = channel_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::default().content(format!("Error fetching menu {e:#?}")),
                        )
                        .await
                    {
                        println!("Error {e:#?}");
                    }
                }
            }
        })
    })
}

pub async fn save_jobs(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let jobs = ctx.data().job_uuids.lock().await;
    let stored_jobs: Vec<StoredJob> = jobs.iter().map(|j| j.into()).collect();

    let data = serde_json::to_string(&stored_jobs)?;
    write("jobs.json", data)?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn schedule_day(
    ctx: Context<'_>,
    #[description = "Cron schedule"] cron: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let channel_id = ctx.channel_id();

    let job = create_scheduled_day_post(&ctx.serenity_context(), &cron, channel_id)?;

    {
        let mut jobs = ctx.data().job_uuids.lock().await;

        jobs.push(DataJob {
            uuid: job.guid(),
            cron: cron.into(),
            channel_id: channel_id.get(),
        });
    }

    save_jobs(ctx).await?;

    let s = ctx.data().sched.lock().await;
    s.add(job).await?;

    ctx.send(CreateReply::default().content("Ajoitettu ruokalistaviesti luotu").ephemeral(true))
        .await?;

    Ok(())
}
