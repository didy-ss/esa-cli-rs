use std::env;

#[derive(Debug)]
pub struct UserConfig {
    pub api_key: String,
    pub team: String,
    pub user: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The environment variable `{0}` is not set")]
    NotFound(String),
}

impl UserConfig {
    pub fn from_env() -> Result<UserConfig, Error> {
        let api_key =
            env::var("ESA_API_KEY").map_err(|_| Error::NotFound("ESA_API_KEY".to_string()))?;
        let team = env::var("ESA_TEAM").map_err(|_| Error::NotFound("ESA_TEAM".to_string()))?;
        let user = env::var("ESA_USER").map_err(|_| Error::NotFound("ESA_USER".to_string()))?;

        Ok(UserConfig {
            api_key,
            team,
            user,
        })
    }
}
