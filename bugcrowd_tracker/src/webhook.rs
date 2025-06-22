use std::{sync::LazyLock, time::Duration};

use log::info;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use twilight_model::{
    channel::message::Embed,
    id::{Id, marker::WebhookMarker},
};

#[derive(Debug, Clone)]
pub struct Webhook {
    pub id: Id<WebhookMarker>,
    pub token: String,
}

pub async fn send_webhook(webhook: &Webhook, embed: Embed) -> Result<(), anyhow::Error> {
    static REQUEST_CLIENT: LazyLock<Client> = LazyLock::new(|| {
        ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .http2_prior_knowledge()
            .build()
            .unwrap()
    });

    let response = REQUEST_CLIENT
        .post(format!(
            "https://discord.com/api/v9/webhooks/{}/{}?wait=true",
            webhook.id, webhook.token
        ))
        .json(&json!({ "embeds": [embed] }))
        .send()
        .await?;
    if response.status() == 429 {
        let retry_after = response
            .headers()
            .get("retry-after")
            .map(|retry| retry.to_str().unwrap().parse::<u64>().unwrap());
        if retry_after.is_none() {
            return Err(anyhow::Error::msg("rate limited, no retry-affter".to_string()));
        }

        let retry_after = retry_after.unwrap();
        info!("webhook rate limited, waiting {retry_after} seconds");

        let retry_after = Duration::from_secs(retry_after);
        tokio::time::sleep(retry_after).await;
        return Box::pin(send_webhook(webhook, embed)).await;
    }

    response.error_for_status()?;
    Ok(())
}

pub mod hall_of_fame {
    use bugcrowd_api::models::Hero;
    use log::{debug, error, info};
    use tokio::sync::mpsc::{Sender, channel};
    use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

    use crate::{
        breakdown,
        poll::hall_of_fame::Event,
        webhook::{Webhook, send_webhook},
    };

    pub fn background_channel(webhook: Webhook) -> Sender<Event> {
        let (sender, mut receiver) = channel(1);
        tokio::task::spawn(async move {
            while let Some(event) = receiver.recv().await {
                info!("recieved event {event:?}");
                if let Err(error) = handle_event(&webhook, event).await {
                    error!("failed to handle event {error:#?}");
                    break;
                }
            }
        });

        sender
    }

    const BUGCROWD_THEME_COLOR: u32 = 15886370;
    const SUCCESS_COLOR: u32 = 5294200;
    const FAILURE_COLOR: u32 = 16711680;

    async fn handle_event(webhook: &Webhook, event: Event) -> Result<(), anyhow::Error> {
        let embed = match event {
            Event::HeroAdded(hero) => {
                let _hero = display(&hero);
                let content = format!(
                    "{_hero} was added to the leaderboard with **{} points** (rank: {})",
                    hero.points, hero.rank
                );
                EmbedBuilder::new()
                    .color(BUGCROWD_THEME_COLOR)
                    .description(content)
                    .validate()?
                    .build()
            }

            Event::HeroRemoved(hero) => {
                let _hero = display(&hero);
                let content = format!(
                    "{_hero} was removed from the leaderboard (rank: {})",
                    hero.rank
                );
                EmbedBuilder::new()
                    .color(FAILURE_COLOR)
                    .description(content)
                    .validate()?
                    .build()
            }

            Event::HeroUpdated(old, new) => {
                if old.points > new.points {
                    // Lost points
                    let lost = old.points - new.points;
                    let _hero = display(&new);
                    let content = format!(
                        "{_hero} lost **-{lost} points** and now has **{} points**",
                        new.points
                    );
                    let mut embed = EmbedBuilder::new()
                        .color(FAILURE_COLOR)
                        .description(content);

                    if old.rank != new.rank {
                        let rank_change = (old.rank as isize) - (new.rank as isize);
                        let footer = format!("#{} -> #{} (-{}))", old.rank, new.rank, rank_change);
                        embed = embed.footer(EmbedFooterBuilder::new(footer));
                    }

                    embed.validate()?.build()
                } else {
                    // Gained points
                    let gained = new.points - old.points;
                    let _hero = display(&new);
                    let breakdown = breakdown::calculate_points_breakdown(gained as i32);

                    let content = if let Some(breakdown) = breakdown {
                        let breakdown = breakdown.to_string();
                        format!(
                            "{_hero} gained **+{gained} points** [{breakdown}] and now has **{} points**",
                            new.points
                        )
                    } else {
                        format!(
                            "{_hero} gained **+{gained} points** and now has **{} points**",
                            new.points
                        )
                    };

                    let mut embed = EmbedBuilder::new()
                        .color(SUCCESS_COLOR)
                        .description(content);

                    if old.rank != new.rank {
                        let rank_change = (old.rank as isize) - (new.rank as isize);
                        let footer = format!("#{} -> #{} (+{})", old.rank, new.rank, rank_change);
                        embed = embed.footer(EmbedFooterBuilder::new(footer));
                    }

                    embed.validate()?.build()
                }
            }
        };

        debug!("sending embed: {embed:?}");
        send_webhook(webhook, embed).await?;
        Ok(())
    }

    fn display(hero: &Hero) -> String {
        if let Some(profile_url) = hero.profile_url.as_ref() {
            format!("[**`{}`**]({})", hero.username, profile_url)
        } else {
            format!("**`{}`**", hero.username)
        }
    }
}
