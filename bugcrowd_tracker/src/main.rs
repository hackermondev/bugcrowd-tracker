use std::time::Duration;

use bugcrowd_api::client::BugcrowdApi;
use clap::Parser;
use log::{error, info};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use twilight_model::id::Id;

use crate::{config::Arguments, store::{DisclosedReportsStore, HallOfFameStore}, webhook::Webhook};

mod breakdown;
mod config;
mod poll;
mod store;
mod webhook;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let config = Arguments::parse();
    let bugcrowd_api = BugcrowdApi::new(config.bugcrowd_session_token);
    let program_handle = config.engagement_handle;
    let webhook =
        extract_webhook(&config.discord_webhook_url).expect("Invalid Discord webhook URL");

    let redis = redis::Client::open(config.redis).expect("Invalid Redis connection URI");
    let redis = redis
        .get_connection_manager()
        .await
        .expect("Failed to open Redis connection");

    let program_access = bugcrowd_api
        .has_program_access(&program_handle)
        .await
        .unwrap();
    if !program_access {
        error!(
            "Unable to fetch program, ensure you're using the program handle, and a session token for private programs"
        );
        return;
    }

    let mut polls = vec![];

    // ==== Hall Of Fame ====
    {
        let store = HallOfFameStore {
            program_handle: program_handle.clone(),
            redis_connection: redis.clone(),
        };
        let channel = webhook::hall_of_fame::background_channel(webhook.clone());
        let mut poller = poll::hall_of_fame::Poller {
            bugcrowd: bugcrowd_api.clone(),
            store,
            program_handle: program_handle.clone(),
            channel,
        };

        const POLL_INTERVAL: Duration = Duration::from_secs(60 * 3);
        polls.push(tokio::task::spawn(async move {
            info!("started polling hall of fame");

            loop {
                poller.poll().await.expect("hall of fame poll failed");
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }));
    }

    // ==== Crowdstream Disclosed Reports ====
    {
        let store = DisclosedReportsStore {
            program_handle: program_handle.clone(),
            redis_connection: redis.clone(),
        };
        let channel = webhook::disclosed_reports::background_channel(webhook.clone());
        let mut poller = poll::disclosed_reports::Poller {
            bugcrowd: bugcrowd_api.clone(),
            store,
            program_handle: program_handle.clone(),
            channel,
        };

        const POLL_INTERVAL: Duration = Duration::from_secs(60 * 5);
        polls.push(tokio::task::spawn(async move {
            info!("started polling Crowdstream disclosed reports");

            loop {
                poller.poll().await.expect("Crowdstream disclosed reports poll failed");
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }));
    }

    let _ = futures::future::select_all(polls).await;
}

fn extract_webhook(url: &str) -> Option<Webhook> {
    let path_parts: Vec<&str> = url.trim_start_matches("https://").split('/').collect();
    if path_parts.len() >= 4 && path_parts[1] == "api" && path_parts[2] == "webhooks" {
        let webhook_id = path_parts[3].parse::<u64>().ok()?;
        let token = path_parts[4];

        Some(Webhook {
            id: Id::new(webhook_id),
            token: token.to_string(),
        })
    } else {
        None
    }
}
