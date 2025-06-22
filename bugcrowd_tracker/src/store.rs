use bugcrowd_api::models::{DisclosedReport, Hero};
use redis::AsyncCommands;

type RedisConnection = redis::aio::ConnectionManager;

// ==== Hall Of Fame ====
pub struct HallOfFameStore {
    pub program_handle: String,
    pub redis_connection: RedisConnection,
}

impl HallOfFameStore {
    fn key(&self) -> String {
        format!("bugcrowd_tracker:/hof/{}", self.program_handle)
    }

    pub async fn heros(&mut self) -> Result<Vec<Hero>, anyhow::Error> {
        let key = self.key();
        let heros: Vec<String> = self.redis_connection.smembers(key).await?;
        let heros = heros
            .into_iter()
            .map(|hero| serde_json::from_str(&hero).unwrap())
            .collect();
        Ok(heros)
    }

    pub async fn save_heros(&mut self, heros: Vec<Hero>) -> Result<(), anyhow::Error> {
        let mut pipeline = redis::pipe();
        let key = self.key();

        pipeline.del(&key).ignore();
        for hero in heros {
            let hero = serde_json::to_string(&hero)?;
            pipeline.sadd(&key, hero).ignore();
        }

        pipeline.exec_async(&mut self.redis_connection).await?;
        Ok(())
    }
}


// ==== Disclosed Reports ====
pub struct DisclosedReportsStore {
    pub program_handle: String,
    pub redis_connection: RedisConnection,
}

impl DisclosedReportsStore {
    fn key(&self) -> String {
        format!("bugcrowd_tracker:/disclosed/{}", self.program_handle)
    }

    pub async fn last_disclosed_report(&mut self) -> Result<Option<DisclosedReport>, anyhow::Error> {
        let key = self.key();
        let report: Option<String> = self.redis_connection.get(key).await?;
        let report = report.map(|report| serde_json::from_str(&report).unwrap());
        Ok(report)
    }

    pub async fn set_last_disclosed_report(&mut self, report: &DisclosedReport) -> Result<(), anyhow::Error> {
        let key = self.key();
        let report = serde_json::to_string(report)?;
        self.redis_connection.set::<_, _, ()>(key, report).await?;
        Ok(())
    }
}