use poise::CreateReply;
use serenity::all::CreateButton;
use serenity::all::ReactionType;
use serenity::{
    all::{Colour, CreateActionRow, CreateEmbed},
    futures,
};

use crate::{Context, Error, types::common::Recipe};

pub mod extra_info;

use crate::types::{common::Course, day::DailyMenu};

const CENTRIA: u8 = 129;

fn fmt_course(course: Course) -> CreateEmbed {
    let title = course.title_fi.unwrap_or("N/A".to_string());
    let price = course.price.unwrap_or("N/A".to_string());

    let mut embed = CreateEmbed::new()
	.title(format!("{} - {}", title, price));

    let category = course.category.unwrap_or("N/A".to_string());
    let diet_info = course.diet_info;
    let food_info = course.additional_diet_info.food_info;

    if category.contains("VEGAN") {
        embed = embed.color(Colour::DARK_GREEN);
    } else if category.contains("bakery") {
        embed = embed.color(Colour::ORANGE);
    } else if category.contains("favorites") {
        embed = embed.color(Colour::RED);
    } else if category.contains("SOUP") {
        embed = embed.color(Colour::TEAL);
    } else if category.contains("SWEET") {
        embed = embed.color(Colour::FABLED_PINK);
    }

    embed = embed.description(format!(
        r#"
- Gluteeniton {}
- Laktoositon {}
- Maidoton {}
- Vähälaktoosinen {}

- =< 0.5 kg CO2 Päästöt {}
- Parempi Valinta {}
- Vegaaninen {}
- Opiskelijaruokailusuositusten mukainen {}

- Sisältää porsaanlihaa {}
- Liha Suomesta {}
- Liha EU:sta {}
- Liha muualta {}
"#,
        if diet_info.gluten_free { "✅" } else { "❌" },
        if diet_info.lactose_free { "✅" } else { "❌" },
        if diet_info.milk_free { "✅" } else { "❌" },
        if diet_info.low_lactose {
            "✅"
        } else {
            "❌"
        },

	if food_info.co2 { "✅" } else { "❌" },
	if food_info.heart { "✅" } else { "❌" },
	if food_info.vegan { "✅" } else { "❌" },
	if food_info.student_recommendation { "✅" } else { "❌" },
	if food_info.pork { "✅" } else { "❌" },
	if food_info.fi_meat { "✅" } else { "❌" },
	if food_info.eu_meat { "✅" } else { "❌" },
	if food_info.other_meat { "✅" } else { "❌" },
    ));

    embed
}

fn fmt_day(day: &str, menu: DailyMenu) -> CreateReply {
    let meta = menu.meta;
    let courses = menu.courses;

    let mut buttons: Vec<CreateButton> = Vec::with_capacity(5);
    let mut reply = CreateReply::default().content(format!(
        r#"
    # [{}]({})

    "#,
        meta.ref_title, meta.ref_url
    ));

    for (n, c) in courses.clone().into_iter() {
        let name = c.title_fi.clone().unwrap_or("N/A".to_string());

        let button = CreateButton::new(format!("infoday_{}_{}", day, n))
            .label(format!("{name}"))
            .emoji(ReactionType::Unicode("ℹ️".to_string()));

        buttons.push(button);

        reply = reply.embed(fmt_course(c));
    }

    // this is a length check for the button vec
    // discord only allows 5 buttons per actionrow, there might theoretically be more
    // TODO figure out a better way
    let mut finalbuttons: Vec<Vec<CreateButton>> = Vec::new();
    let mut acrs: Vec<CreateActionRow> = Vec::new();

    {
        while buttons.len() > 5 {
            finalbuttons.push(buttons.drain(0..5).collect());
        }

        finalbuttons.push(buttons);

        for i in finalbuttons {
            let acr = CreateActionRow::Buttons(i);
            acrs.push(acr);
        }
    }

    reply.components(acrs)
}

#[poise::command(slash_command)]
pub async fn ruokalista(
    ctx: Context<'_>,
    #[description = "Päivämäärä (YYYY-MM-DD)"] day: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let day = match day {
        Some(day) => day,
        None => chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };

    let menu = fetch_day(&day).await?;

    let reply = fmt_day(&day, menu);

    // send the message
    ctx.send(reply.ephemeral(true)).await?;

    Ok(())
}

pub async fn fetch_day(day: &str) -> Result<DailyMenu, Error> {
    let url = format!(
        "https://sodexo.fi/ruokalistat/output/daily_json/{}/{}",
        CENTRIA, day
    );

    let menu = reqwest::get(url).await?.json::<DailyMenu>().await?;

    Ok(menu)
}
