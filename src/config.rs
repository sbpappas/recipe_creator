use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub app_secret: String,
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8090".into())
            .parse()
            .map_err(|_| "PORT must be a valid number".to_string())?;

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/recipe_creator.db?mode=rwc".into());

        let app_secret = env::var("APP_SECRET")
            .map_err(|_| "APP_SECRET is required (use a long random string)".to_string())?;

        if app_secret.len() < 16 {
            return Err("APP_SECRET must be at least 16 characters".to_string());
        }

        let gemini_api_key = env::var("GEMINI_API_KEY").ok().filter(|k| !k.is_empty());
        let gemini_model =
            env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".into());

        Ok(Self {
            port,
            database_url,
            app_secret,
            gemini_api_key,
            gemini_model,
        })
    }
}
