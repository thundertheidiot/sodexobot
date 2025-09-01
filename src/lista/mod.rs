use crate::types::week::WeeklyMenu;
use chrono::Datelike;
use chrono::Days;
use poise::CreateReply;
use serenity::all::CreateButton;
use serenity::all::ReactionType;
use serenity::all::{Colour, CreateActionRow, CreateEmbed};

use crate::{Context, Error, types::common::Recipe};

pub mod extra_info;

use crate::types::{common::Course, day::DailyMenu};

fn fmt_course(course: Course) -> CreateEmbed {
    let title = course.title_fi.unwrap_or("N/A".to_string());
    let price = course.price.unwrap_or("N/A".to_string());

    let mut embed = CreateEmbed::new().title(title);

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
        r"
Hinta: `{price}`
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
- Liha muualta EU:sta {}
- Liha muualta {}
",
        if diet_info.gluten_free { "✅" } else { "❌" },
        if diet_info.lactose_free { "✅" } else { "❌" },
        if diet_info.milk_free { "✅" } else { "❌" },
        if diet_info.low_lactose { "✅" } else { "❌" },
        if food_info.co2 { "✅" } else { "❌" },
        if food_info.heart { "✅" } else { "❌" },
        if food_info.vegan { "✅" } else { "❌" },
        if food_info.student_recommendation {
            "✅"
        } else {
            "❌"
        },
        if food_info.pork { "✅" } else { "❌" },
        if food_info.fi_meat { "✅" } else { "❌" },
        if food_info.eu_meat { "✅" } else { "❌" },
        if food_info.other_meat { "✅" } else { "❌" },
    ));

    embed
}

pub fn fmt_day(day: &str, menu: DailyMenu, extra_string: Option<&str>) -> CreateReply {
    let meta = menu.meta;
    let courses = menu.courses;

    match courses.len() {
        n if n > 0 => {
            let mut buttons: Vec<CreateButton> = Vec::with_capacity(5);
            let mut reply = CreateReply::default().content(format!(
                r"
    # [{}](<{}>) - {day}
    {}",
                meta.ref_title,
                meta.ref_url,
                extra_string.unwrap_or_default()
            ));

            for (n, c) in courses {
                let name = c.title_fi.clone().unwrap_or("N/A".to_string());

                let mut button = CreateButton::new(format!("infoday_{day}_{n}"))
                    .emoji(ReactionType::Unicode("ℹ️".to_string()));

                match name.len() {
                    n if n >= 80 => {
                        let name = &name[0..77];
                        let name = name.to_string() + "...";
                        button = button.label(name);
                    }
                    _ => {
                        button = button.label(name);
                    }
                }

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
        _ => CreateReply::default().content(format!(
            r"
    # [{}](<{}>)

    Ei ruokalistaa päivälle {day}
    ",
            meta.ref_title, meta.ref_url
        )),
    }
}

#[poise::command(slash_command)]
pub async fn ruokalista(
    ctx: Context<'_>,
    #[description = "Päivämäärä (YYYY-MM-DD)"] day: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let day = match day {
        Some(day) => day,
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

#[poise::command(slash_command)]
pub async fn viikon_lista(ctx: Context<'_>) -> Result<(), Error> {
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
            .ok_or("what the fuck")?
            .format("%Y-%m-%d")
            .to_string();

        let reply = fmt_day(&day, m, Some(&n));
        ctx.send(reply).await?;
    }

    Ok(())
}

const CENTRIA: u8 = 129;

pub async fn fetch_day(day: &str) -> Result<DailyMenu, Error> {
    let url = format!("https://sodexo.fi/ruokalistat/output/daily_json/{CENTRIA}/{day}");

    let menu = reqwest::get(url).await?.json::<DailyMenu>().await?;

    Ok(menu)
}

pub async fn fetch_week() -> Result<WeeklyMenu, Error> {
    let url = format!("https://sodexo.fi/ruokalistat/output/weekly_json/{CENTRIA}",);

    let menu = reqwest::get(url).await?.json::<WeeklyMenu>().await?;

    Ok(menu)
}
