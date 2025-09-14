use chrono::Datelike;
use crate::types::day::DailyMenu;
use crate::list::fetch_week;
use crate::list::fmt_day;
use crate::list::fetch_day;
use chrono::Days;
use crate::{Context, Error};

#[poise::command(
    slash_command,
    rename = "ruokalista",
)]
pub async fn daily_menu(
    ctx: Context<'_>,
    #[description = "Päivämäärä (YYYY-MM-DD) tai offset (+n) päivää"] day: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let day = match day {
        Some(day) => {
            if &day[0..1] == "+" {
                let n = &day[1..].parse::<u64>()?;
                let date = chrono::Local::now();

                date.checked_add_days(Days::new(*n))
                    .ok_or("invalid number of days")?
                    .format("%Y-%m-%d")
                    .to_string()
            } else {
                day
            }
        }
        None => chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };

    let menu = fetch_day(&day).await?;

    let reply = fmt_day(&day, menu, None);

    // send the message
    ctx.send(reply.ephemeral(true)).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    rename = "viikon-ruokalista"
)]
pub async fn weekly_menu(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let menu = fetch_week().await?;
    let date: Vec<&str> = menu.timeperiod.split('.').collect();
    let day = date.first().ok_or("invalid date in json")?;

    // TODO this is garbage
    let date = chrono::Local::now()
        .with_day(day.parse()?)
        .ok_or("invalid day")?;

    let menus: Vec<(String, DailyMenu)> = menu.into();

    for (i, (n, m)) in menus.into_iter().enumerate() {
        let day = date
            .checked_add_days(Days::new(i as u64))
            .ok_or("day somehow invalid")?
            .format("%Y-%m-%d")
            .to_string();

        let reply = fmt_day(&day, m, Some(&n));
        ctx.send(reply).await?;
    }

    Ok(())
}
