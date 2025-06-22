use chrono::{DateTime, Utc};
use dateparser::DateTimeUtc;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationMeta {
    pub limit: u64,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
}

// ==== Hall Of Fame ====
#[derive(Serialize, Deserialize, Debug)]
pub struct HallOfFame {
    pub pagination_meta: PaginationMeta,
    pub presented_heroes: HallOfFameHeros,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HallOfFameHeros {
    pub heroes: Vec<Hero>,
    pub hide_points: bool,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hero {
    #[serde(rename = "accentColor")]
    pub accent_color: Option<String>,
    pub avatar: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub points: u64,
    #[serde(rename = "profileUrl")]
    pub profile_url: Option<String>,
    pub username: String,
    #[serde(default)]
    pub rank: usize,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

// ==== Crowdstream ====
#[derive(Serialize, Deserialize, Debug)]
pub struct CrowdStream {
    pub pagination_meta: PaginationMeta,
    pub results: Vec<DisclosedReport>
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DisclosedReport {
    pub id: String,
    pub title: String,
    pub target: Option<String>,
    pub researcher_username: Option<String>,
    pub amount: Option<String>,
    pub priority: u32,
    #[serde(deserialize_with = "deserialize_common_date_formats")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_common_date_formats")]
    pub disclosed_at: DateTime<Utc>,
    pub disclosure_report_url: String,
}

fn deserialize_common_date_formats<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let datetime = String::deserialize(deserializer)?;
    let parsed = datetime.parse::<DateTimeUtc>().unwrap();
    Ok(parsed.0)
}
