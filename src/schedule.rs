use crate::lista::fetch_day;
use crate::lista::fmt_day;
use crate::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude::CreateMessage;
use serenity::all::MessageReference;
use serenity::all::MessageReferenceKind;
use std::sync::Arc;
use tokio_cron_scheduler::Job;

#[poise::command(slash_command)]
pub async fn schedule_day(
    ctx: Context<'_>,
    #[description = "Cron schedule"] cron: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let serenity_ctx = ctx.serenity_context().clone();
    let serenity_ctx = Arc::new(serenity_ctx);
    let channel_id = ctx.channel_id();

    let job = Job::new_async(cron, move |_uuid, _l| {
        let ctx = serenity_ctx.clone();

        Box::pin(async move {
            println!("cid {}", channel_id);

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
    })?;

    let s = ctx.data().sched.lock().await;
    s.add(job).await?;

    ctx.send(CreateReply::default().content("ligma").ephemeral(true))
        .await?;

    Ok(())
}
