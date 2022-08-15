use anyhow::Result;
use serde_json::{json, Value};
use std::fs::metadata;
use std::path::PathBuf;
use std::process::Command;

use crate::user_config::UserConfig;

const ESA_ENDPOINT: &str = "https://api.esa.io";

pub struct EsaClient {
    user_config: UserConfig,
    client: reqwest::blocking::Client,
}

pub enum FetchParameter {
    Qstring(Vec<String>),
    PathNumber(u64),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request error: {0}")]
    RequestSend(#[from] reqwest::Error),

    #[error("json error: {0}")]
    ResponseJson(#[from] serde_json::Error),

    #[error("curl error: {0}")]
    Curl(#[from] std::io::Error),
}

impl EsaClient {
    pub fn new(user_config: UserConfig) -> EsaClient {
        EsaClient {
            user_config,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn create_post(self, params: Value) -> Result<Value, Error> {
        Ok(self
            .client
            .post(format!(
                "{}/v1/teams/{}/posts",
                ESA_ENDPOINT, self.user_config.team
            ))
            .header(
                "Authorization",
                format!("Bearer {}", self.user_config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&params)
            .send()?
            .json()?)
    }

    pub fn update_post(self, post_number: u64, params: Value) -> Result<Value, Error> {
        Ok(self
            .client
            .patch(format!(
                "{}/v1/teams/{}/posts/{}",
                ESA_ENDPOINT, self.user_config.team, post_number
            ))
            .header(
                "Authorization",
                format!("Bearer {}", self.user_config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&params)
            .send()?
            .json()?)
    }

    pub fn fetch_post(self, parameter: FetchParameter) -> Result<Value, Error> {
        let client = self.client;
        Ok(match parameter {
            FetchParameter::Qstring(q_string) => {
                let q_string = q_string.join(" ");
                client.get(format!(
                    "{}/v1/teams/{}/posts?q={}",
                    ESA_ENDPOINT, self.user_config.team, q_string
                ))
            }
            FetchParameter::PathNumber(number) => client.get(format!(
                "{}/v1/teams/{}/posts/{}",
                ESA_ENDPOINT, self.user_config.team, number
            )),
        }
        .header(
            "Authorization",
            format!("Bearer {}", self.user_config.api_key),
        )
        .header("Content-Type", "application/json")
        .send()?
        .json()?)
    }

    pub fn upload_image(self, image_file: PathBuf) -> Result<String, Error> {
        let policies = self.create_attachment_policies(image_file.clone())?;
        self.upload_image_to_s3(image_file, policies)
    }

    fn create_attachment_policies(&self, image_file: PathBuf) -> Result<Value, Error> {
        Ok(self
            .client
            .post(format!(
                "{}/v1/teams/{}/attachments/policies",
                ESA_ENDPOINT, self.user_config.team
            ))
            .header(
                "Authorization",
                format!("Bearer {}", self.user_config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&json!(
                {
                    "type": mime_guess::from_path(image_file.clone()).first().unwrap().to_string(),
                    "size": metadata(image_file.clone()).unwrap().len(),
                    "name": image_file.to_string_lossy(),
                }
            ))
            .send()?
            .json()?)
    }

    fn upload_image_to_s3(&self, image_file: PathBuf, policies: Value) -> Result<String, Error> {
        let output = Command::new("curl")
            .args(&[
                "-F",
                &format!("key={}", policies["form"]["key"].as_str().unwrap()),
                "-F",
                &format!(
                    "AWSAccessKeyId={}",
                    policies["form"]["AWSAccessKeyId"].as_str().unwrap()
                ),
                "-F",
                &format!(
                    "signature={}",
                    policies["form"]["signature"].as_str().unwrap()
                ),
                "-F",
                &format!("policy={}", policies["form"]["policy"].as_str().unwrap()),
                "-F",
                &format!(
                    "Content-Type={}",
                    policies["form"]["Content-Type"].as_str().unwrap()
                ),
                "-F",
                &format!(
                    "Content-Disposition={}",
                    policies["form"]["Content-Disposition"].as_str().unwrap()
                ),
                "-F",
                &format!(
                    "Cache-Control={}",
                    policies["form"]["Cache-Control"].as_str().unwrap()
                ),
                "-F",
                &format!("acl={}", policies["form"]["acl"].as_str().unwrap()),
                "-F",
                &format!("file=@{}", image_file.to_string_lossy()),
                "--include",
                policies["attachment"]["endpoint"].as_str().unwrap(),
            ])
            .output()?;

        Ok(String::from_utf8(output.stdout).unwrap())
    }
}
