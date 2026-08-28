use std::{pin::Pin, sync::Arc, time::Duration};

use bugcrowd_api::client::BugcrowdApi;
use clap::Parser;
use log::{error, info};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use twilight_model::id::Id;

use crate::{
    config::Arguments,
    store::{disclosed_reports::DisclosedReportsStore, hall_of_fame::HallOfFameStore},
    webhook::Webhook,
};

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
    let program_handle = Arc::<str>::from(config.engagement_handle);
    let webhook =
        extract_webhook(&config.discord_webhook_url).expect("Invalid Discord webhook URL");
    let blacklisted_users = Arc::new(config.blacklist_users);

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

    let hall_of_fame_poller = Box::pin(async {
        let store = HallOfFameStore {
            program_handle: program_handle.clone(),
            redis_connection: redis.clone(),
        };
        let channel = webhook::hall_of_fame::background_channel(webhook.clone());
        let mut poller = poll::hall_of_fame::Poller {
            bugcrowd: bugcrowd_api.clone(),
            store,
            channel,
            program_handle: program_handle.clone(),
            blacklisted_users: blacklisted_users.clone(),
        };

        const POLL_INTERVAL: Duration = Duration::from_secs(60 * 3);
        info!("started polling hall of fame");

        loop {
            let poll_result = poller.poll().await;
            if poll_result.is_err() {
                poll_result.expect("Hall of fame polling failed");
                break;
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }) as Pin<Box<dyn Future<Output = ()>>>;

    let crowdstream_disclosed_reports_poller = Box::pin(async {
        let store = DisclosedReportsStore {
            program_handle: program_handle.clone(),
            redis_connection: redis.clone(),
        };
        let channel = webhook::disclosed_reports::background_channel(webhook.clone());
        let mut poller = poll::disclosed_reports::Poller {
            bugcrowd: bugcrowd_api.clone(),
            store,
            channel,
            program_handle: program_handle.clone(),
            blacklisted_users: blacklisted_users.clone(),
        };

        const POLL_INTERVAL: Duration = Duration::from_secs(60 * 5);
        info!("started polling Crowdstream disclosed reports");

        loop {
            let poll_result = poller.poll().await;
            if poll_result.is_err() {
                poll_result.expect("Crowdstream disclosed reports poll failed");
                break;
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }) as Pin<Box<dyn Future<Output = ()>>>;

    futures::future::join_all([hall_of_fame_poller, crowdstream_disclosed_reports_poller]).await;
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
