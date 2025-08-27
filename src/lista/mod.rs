use poise::CreateReply;
use serenity::futures;

use crate::{Context, Error};

mod structs;

use structs::DailyMenu;

// pub fn autocomplete_place<'a>(
//     _ctx: Context<'_>,
//     partial: &'a str,
// ) -> impl Stream<Item = String> + 'a {
//     futures::stream::iter
// }

const CENTRIA: u8 = 129;

#[poise::command(slash_command)]
pub async fn ruokalista(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(
	CreateReply::default()
	    .content("ligma balls")
    .ephemeral(true),
    ).await?;

    Ok(())
}

pub async fn fetch_day(
    day: Option<String>,
) -> Result<DailyMenu, Error> {

    let url = format!("https://sodexo.fi/ruokalistat/output/daily_json/{}/{}", CENTRIA, "2025-08-27");

    let menu = reqwest::get(url)
	.await?
	.json::<DailyMenu>()
	.await?;

    Ok(menu)
}
