use poise::CreateReply;
use serenity::all::CreateButton;
use serenity::all::ReactionType;
use serenity::{
    all::{Colour, CreateActionRow, CreateEmbed},
    futures,
};

use crate::{Context, Error, lista::structs::Recipe};

mod structs;
pub mod extra_info;

use structs::{Course, DailyMenu};

// pub fn autocomplete_place<'a>(
//     _ctx: Context<'_>,
//     partial: &'a str,
// ) -> impl Stream<Item = String> + 'a {
//     futures::stream::iter
// }

const CENTRIA: u8 = 129;

struct DietInfo {
    gluteeniton: bool,
    laktoositon: bool,
    maidoton: bool,
    vahalaktoosinen: bool,
}

fn to_diet_info(dietcodes: String) -> DietInfo {
    let mut diet_info = DietInfo {
        gluteeniton: false,
        laktoositon: false,
        maidoton: false,
        vahalaktoosinen: false,
    };

    let codes: Vec<&str> = dietcodes.split(',').collect();

    for i in codes {
        match i {
            "G" => diet_info.gluteeniton = true,
            "L" => diet_info.laktoositon = true,
            "M" => diet_info.maidoton = true,
            "VL" => diet_info.vahalaktoosinen = true,
            _ => (),
        }
    }

    diet_info
}

fn fmt_course(course: Course) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(course.title_fi.unwrap_or("N/A".to_string()));

    let category = course.category.unwrap_or("N/A".to_string());
    let diet_info = to_diet_info(course.dietcodes.unwrap_or("".to_string()));

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
Gluteeniton {}
Laktoositon {}
Maidoton {}
Vähälaktoosinen {}

Hinta {}
"#,
        if diet_info.gluteeniton { "✅" } else { "❌" },
        if diet_info.laktoositon { "✅" } else { "❌" },
        if diet_info.maidoton { "✅" } else { "❌" },
        if diet_info.vahalaktoosinen {
            "✅"
        } else {
            "❌"
        },
        course.price.unwrap_or("N/A".to_string()),
        // if course.recipes.is_some() {
        //     let recipes = course.recipes.unwrap();

        //     fmt_recipe(recipes.recipes.first().unwrap())
        // } else { "".to_string() }
    ));

    embed
}

#[poise::command(slash_command)]
pub async fn ruokalista(
    ctx: Context<'_>,
    #[description = "Päivämäärä (YYYY-MM-DD)"] day: Option<String>,
) -> Result<(), Error> {
    let day = match day {
        Some(day) => day,
        None => chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    };

    let menu = fetch_day(&day).await?;
    let meta = menu.meta;
    let courses = menu.courses;

    //     let mut reply = CreateReply::default().content(format!(r#"
    // # [{}]({})

    // "#, meta.ref_title, meta.ref_url));

    for (n, c) in courses.into_iter() {
        let button = CreateButton::new(format!("infoday_{}_{}", day, n))
            .label("Lisätietoja")
            .emoji(ReactionType::Unicode("ℹ️".to_string()));
        let acr = CreateActionRow::Buttons(vec![button]);

        ctx.send(
            CreateReply::default()
                .content(format!(
                    r#"
# [{}]({})
"#,
                    meta.ref_title, meta.ref_url
                ))
                .embed(fmt_course(c))
                .components(vec![acr])
                .ephemeral(true),
        )
        .await?;
    }

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
