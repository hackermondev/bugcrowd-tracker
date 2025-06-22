use clap::Parser;

#[derive(Default, Debug, Parser)]
#[clap(author = "daniel", version, about)]
pub struct Arguments {
    #[arg(short = 'T', long = "session-token", env = "SESSION_TOKEN")]
    pub bugcrowd_session_token: Option<String>,
    #[arg(short = 'R', long = "redis", env = "REDIS_URL")]
    pub redis: String,
    #[arg(short = 'H', long = "handle")]
    pub engagement_handle: String,
    #[arg(short = 'W', long = "webhook-url")]
    pub discord_webhook_url: String,
}
