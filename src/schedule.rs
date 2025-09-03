use crate::list::fetch_day;
use crate::list::fmt_day;
use crate::{Context, Error};
use chrono::Local;
use chrono_tz::Europe::Helsinki;
use poise::serenity_prelude::CreateMessage;
use serde::Deserialize;
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

#[derive(Debug)]
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

impl From<&DataJob> for StoredJob {
    fn from(val: &DataJob) -> Self {
        StoredJob {
            cron: val.cron.clone(),
            channel_id: val.channel_id,
        }
    }
}

pub fn create_scheduled_day_post<S: ToString>(
    ctx: &poise::serenity_prelude::Context,
    cron: S,
    channel_id: ChannelId,
) -> Result<JobLocked, JobSchedulerError> {
    let ctx = Arc::new(ctx.clone());

    Job::new_async_tz(cron, Helsinki, move |_uuid, _l| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let day = Local::now().date_naive().format("%Y-%m-%d").to_string();

            match fetch_day(&day).await {
                Ok(menu) => {
                    let reply = fmt_day(&day, menu, None);

                    let m = reply.to_prefix(MessageReference::new(
                        MessageReferenceKind::Default,
                        channel_id,
                    ));
                    if let Err(e) = channel_id.send_message(&ctx.http, m).await {
                        println!("Error sending message {e:#?}");
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
                        println!("unable to send error message {e:#?}");
                    }
                }
            }
        })
    })
}

pub async fn save_jobs(ctx: Context<'_>) -> Result<(), Error> {
    let jobs = ctx.data().job_uuids.lock().await;
    let stored_jobs: Vec<StoredJob> = jobs.iter().map(std::convert::Into::into).collect();

    let data = serde_json::to_string(&stored_jobs)?;
    write("jobs.json", data)?;

    Ok(())
}
