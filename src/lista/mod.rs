use poise::CreateReply;
use serenity::all::CreateButton;
use serenity::all::ReactionType;
use serenity::{
    all::{Colour, CreateActionRow, CreateEmbed},
    futures,
};

use crate::{Context, Error, lista::structs::Recipe};

pub mod extra_info;
mod structs;

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

struct FoodInfo {
    co2: bool,
    heart: bool,
    vegan: bool,
    student_recommendation: bool,
    pork: bool,
    fi_meat: bool,
    eu_meat: bool,
    other_meat: bool,
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

fn to_food_info(images: Vec<String>) -> FoodInfo {
    let mut food_info = FoodInfo {
        co2: false,
        heart: false,
        vegan: false,
        student_recommendation: false,
        pork: false,
        fi_meat: false,
        eu_meat: false,
        other_meat: false,
    };

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/sodexo-leaf.svg".to_string()) {
	food_info.co2 = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/sydan.svg".to_string()) {
	food_info.heart = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/vege.svg".to_string()) {
	food_info.vegan = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/omena.svg".to_string()) {
	food_info.student_recommendation = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/possu.svg".to_string()) {
	food_info.pork = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-fi-new.svg".to_string()) {
	food_info.fi_meat = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-eu-new.svg".to_string()) {
	food_info.eu_meat = true;
    }

    if images.contains(&"https://www.sodexo.fi/sites/default/themes/sodexo/images/liha-muu-new.svg".to_string()) {
	food_info.other_meat = true;
    }

    food_info
}

fn fmt_course(course: Course) -> CreateEmbed {
    let title = course.title_fi.unwrap_or("N/A".to_string());
    let price = course.price.unwrap_or("N/A".to_string());

    let mut embed = CreateEmbed::new()
	.title(format!("{} - {}", title, price));

    let category = course.category.unwrap_or("N/A".to_string());
    let diet_info = to_diet_info(course.dietcodes.unwrap_or("".to_string()));
    let food_info = to_food_info(course.additionalDietInfo.dietcodeImages.unwrap_or(vec![]));

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
        if diet_info.gluteeniton { "✅" } else { "❌" },
        if diet_info.laktoositon { "✅" } else { "❌" },
        if diet_info.maidoton { "✅" } else { "❌" },
        if diet_info.vahalaktoosinen {
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

    // send the message
    ctx.send(reply.ephemeral(true).components(acrs)).await?;

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
