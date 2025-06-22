use bugcrowd_api::models::Hero;
use redis::AsyncCommands;

type RedisConnection = redis::aio::ConnectionManager;

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
