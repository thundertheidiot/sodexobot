use crate::CreateInteractionResponseFollowup;
use crate::Error;
use crate::lista::Recipe;
use crate::lista::fetch_day;
use ::serenity::all::CreateEmbed;
use poise::serenity_prelude as serenity;
use serenity::all::ComponentInteraction;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;

fn fmt_recipe(recipe: &Recipe) -> String {
    format!(
        r#"
## {}

### Ainesosat
```
{}
```
### Ravintosisältö
```
{}
```
"#,
        recipe.name,
        recipe.ingredients.to_string(),
        recipe
            .nutrients
            .split("|")
            .map(|s| s.trim_start())
            .collect::<Vec<&str>>()
            .join("\n")
    )
}

pub async fn extra_info(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let id = &interaction.data.custom_id;
    println!("{}", id);

    let defer = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default());
    interaction.create_response(&ctx.http, defer).await?;

    let mut info: Vec<&str> = id.split('_').collect();
    let n = info.pop().ok_or("cannot get n")?;
    let day = info.pop().ok_or("cannot get day")?;

    let menu = fetch_day(day).await?;
    let course = menu.courses.get(n).ok_or("incorrect n")?.to_owned();

    let allergens = course
        .additional_diet_info
        .allergens
        .unwrap_or("N/A".to_string());

    let recipes = course
        .recipes
        .ok_or("invalid recipe json")?
        .recipes
        .iter()
        .map(|r| fmt_recipe(&r))
        .collect::<Vec<String>>()
        .join("");

    let embed = CreateEmbed::default()
        .title("Reseptit")
        .description(recipes);

    let title = course.title_fi.unwrap_or("N/A".to_string());
    let price = course.price.unwrap_or("N/A".to_string());
    let diet_info = course.diet_info;
    let food_info = course.additional_diet_info.food_info;

    let text = format!(
        r#"
# {title} - {price}

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
# Allergeenit
```
{}
```
"#,
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
        allergens,
    );

    let followup = CreateInteractionResponseFollowup::new()
        .content(text)
        .embed(embed)
        .ephemeral(true);

    interaction.create_followup(&ctx.http, followup).await?;

    Ok(())
}
