use crate::CreateInteractionResponseFollowup;
use crate::Error;
use crate::lista::Recipe;
use crate::lista::fetch_day;
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

    let text = course
        .recipes
        .ok_or("invalid recipe json")?
        .recipes
        .iter()
        .map(|r| fmt_recipe(&r))
        .collect::<Vec<String>>()
        .join("");

    let text = format!(
        r#"
# Allergeenit
```
{}
```
{}
"#,
        allergens, text
    );

    let followup = CreateInteractionResponseFollowup::new()
        .content(text)
        .ephemeral(true);

    interaction.create_followup(&ctx.http, followup).await?;

    Ok(())
}
