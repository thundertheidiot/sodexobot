use poise::serenity_prelude as serenity;
use ::serenity::all::Interaction;
use crate::{Error, Data};
use crate::list::extra_info::extra_info;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::InteractionCreate { interaction } = event
        && let Interaction::Component(c) = interaction
    {
        let id = &c.data.custom_id;

        if id.starts_with("infoday") {
            extra_info(ctx, c).await?;
        }
    }

    Ok(())
}
