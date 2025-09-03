use crate::Context;
use crate::Error;
use crate::schedule::{DataJob, create_scheduled_day_post, save_jobs};
use poise::CreateReply;
use uuid::Uuid;

/// Ajastaa päivän ruokalistaviestin
/// Ajastus noudattaa cron formaattia ja tukee myös sekunteja, eli
///
/// sec min hour day-of-month month day-of-week
/// *   *   *    *            *     *
/// Esimerkiksi 0 0 7 * * mon,tue,wed,thu,fri
/// Lähettää viestin joka viikonpäivänä kello 7 aamulla
#[poise::command(
    slash_command,
    required_permissions = "SEND_MESSAGES | MANAGE_MESSAGES"
)]
pub async fn schedule_day(
    ctx: Context<'_>,
    #[description = "Cron ajastus"] cron: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let channel_id = ctx.channel_id();

    let msg = format!("Ajoitettu ruokalista luotu ajastuksella {cron}");

    let job = create_scheduled_day_post(ctx.serenity_context(), &cron, channel_id)?;

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

    ctx.send(
        CreateReply::default()
            // work around borrow checker
            .content(msg)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "SEND_MESSAGES | MANAGE_MESSAGES"
)]
pub async fn list_scheduled(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let channel_id = ctx.channel_id().get();

    let jobs = ctx.data().job_uuids.lock().await;
    fn fmt_job(job: &DataJob) -> String {
        format!("`{}` - `{}`", job.uuid, job.cron)
    }

    let mut jobs = jobs
        .iter()
        .filter_map(|j| {
            if j.channel_id == channel_id {
                Some(fmt_job(j))
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    if jobs.is_empty() {
        jobs = "Ei ajastettuja ruokalistoja".into();
    }

    ctx.send(CreateReply::default().content(jobs).ephemeral(true))
        .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "SEND_MESSAGES | MANAGE_MESSAGES"
)]
pub async fn delete_scheduled(
    ctx: Context<'_>,
    #[description = "Uuid of scheduled job"] uuid: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let uuid = Uuid::parse_str(&uuid)?;

    {
        let sched = ctx.data().sched.lock().await;
        sched.remove(&uuid).await?;
    }

    {
        let mut jobs = ctx.data().job_uuids.lock().await;
        jobs.retain(|job| job.uuid != uuid);
    }

    save_jobs(ctx).await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Poistettu ajastettu ruokalista `{uuid}` onnistuneesti"
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
