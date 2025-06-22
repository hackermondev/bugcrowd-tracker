use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub points: u64,
    #[serde(rename = "profileUrl")]
    pub profile_url: Option<String>,
    pub username: String,
    #[serde(default)]
    pub rank: usize,
}
