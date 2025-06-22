use std::time::Duration;

use log::{debug, error};
use reqwest::{
    Client, ClientBuilder, Method,
    header::{HeaderMap, HeaderValue},
};

use crate::models::{CrowdStream, DisclosedReport, HallOfFame, Hero};

#[derive(Debug, Clone)]
pub struct BugcrowdApi {
    request_client: Client,
}

impl BugcrowdApi {
    pub fn new(session_token: Option<String>) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "user-agent",
            HeaderValue::from_static("BugcrowdTracker/1.0"),
        );
        if let Some(session_token) = session_token {
            default_headers.insert(
                "Cookie",
                HeaderValue::from_str(&format!("_bugcrowd_session={session_token}")).unwrap(),
            );
        }

        let client = ClientBuilder::new()
            .default_headers(default_headers)
            .http2_prior_knowledge()
            .build()
            .unwrap();

        {
            let client = client.clone();
            tokio::task::spawn(async move { __keepalive_session_background(client).await });
        }

        Self {
            request_client: client,
        }
    }

    pub async fn has_program_access(&self, program: &str) -> Result<bool, anyhow::Error> {
        let url = format!("https://bugcrowd.com/engagements/{program}/announcements.json");
        let response = self.request_client.get(url).send().await?;
        Ok(response.status().is_success())
    }

    pub async fn hall_of_fame(&self, program: &str) -> Result<Vec<Hero>, anyhow::Error> {
        let mut heros = vec![];
        let mut page = 1;

        loop {
            let url = format!(
                "https://bugcrowd.com/engagements/{program}/hall_of_fames.json?page={page}"
            );
            let response = self.request_client.get(url).send().await?;
            let response = response.error_for_status()?;

            let mut hof = response.json::<HallOfFame>().await?;
            heros.append(&mut hof.presented_heroes.heroes);
            page += 1;

            if (hof.pagination_meta.total_count == heros.len() as u64)
                || hof.presented_heroes.total == 0
            {
                break;
            }
        }

        heros.sort_by_key(|h| h.points);
        heros.reverse();
        let heros = heros
            .into_iter()
            .enumerate()
            .map(|(rank, hero)| Hero {
                rank: rank + 1,
                ..hero
            })
            .collect();
        Ok(heros)
    }

    pub async fn last_disclosed_report(
        &self,
        program: &str,
    ) -> Result<Option<DisclosedReport>, anyhow::Error> {
        let url = format!(
            "https://bugcrowd.com/engagements/{program}/crowdstream.json?page=1&filter_by=disclosures"
        );
        let response = self.request_client.get(url).send().await?;
        let response = response.error_for_status()?;

        let crowdstream = response.json::<CrowdStream>().await?;
        let mut reports = crowdstream.results;
        Ok(if !reports.is_empty() {
            Some(reports.remove(0))
        } else {
            None
        })
    }

    pub async fn disclosed_reports_after(
        &self,
        program: &str,
        id: &str,
    ) -> Result<Vec<DisclosedReport>, anyhow::Error> {
        let mut reports = vec![];
        let mut page = 1;

        loop {
            let url = format!(
                "https://bugcrowd.com/engagements/{program}/crowdstream.json?page={page}&filter_by=disclosures"
            );
            let response = self.request_client.get(url).send().await?;
            let response = response.error_for_status()?;

            let crowdstream = response.json::<CrowdStream>().await?;
            let range_end = crowdstream.results.iter().position(|r| r.id == id);
            let found_initial_report = range_end.is_some();

            let range_end = range_end.unwrap_or(crowdstream.results.len());
            reports.append(&mut (crowdstream.results[..range_end]).to_vec());
            page += 1;

            if crowdstream.pagination_meta.total_count == reports.len() as u64
                || found_initial_report
            {
                break;
            }
        }

        Ok(reports)
    }
}

async fn __keepalive_session_background(request_client: Client) {
    const PING_SESSION_INTERVAL: Duration = Duration::from_secs(60);
    let request = request_client
        .request(
            Method::GET,
            "https://bugcrowd.com/auth/session?update_activity=true",
        )
        .build()
        .unwrap();

    loop {
        let request = request.try_clone().unwrap();
        if let Err(session_ping_error) = request_client.execute(request).await {
            error!("failed to keep session alive: {session_ping_error:#?}");
        } else {
            debug!("pinged session")
        }

        tokio::time::sleep(PING_SESSION_INTERVAL).await;
    }
}
