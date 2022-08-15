use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::PathBuf;

use crate::cli::{FetchCommand, SubCommands};
use crate::esa_client::{EsaClient, FetchParameter};
use crate::post::Post;
use crate::user_config::UserConfig;

pub fn run(subcommand: SubCommands, user_config: UserConfig) -> Result<()> {
    match subcommand {
        SubCommands::Create { post_name } => create(post_name),
        SubCommands::Push { post_name } => push(post_name, user_config),
        SubCommands::Fetch(fetch_command) => fetch(fetch_command, user_config),
        SubCommands::Attach { image_file } => attach(image_file, user_config),
    }
}

fn create(post_name: PathBuf) -> Result<()> {
    let post = Post::new(post_name).map_err(|e| anyhow!(e))?;

    post.save().map_err(|e| anyhow!(e))?;

    println!("ok: {}", post.fullname.to_string_lossy());
    Ok(())
}

fn push(post_name: PathBuf, user_config: UserConfig) -> Result<()> {
    let mut post = Post::from_file(post_name).map_err(|e| anyhow!(e))?;

    let esa_client = EsaClient::new(user_config);
    let params = post.to_json();

    let res_body = if let Some(number) = post.meta.number {
        esa_client
            .update_post(number, params)
            .map_err(|e| anyhow!(e))
    } else {
        esa_client.create_post(params).map_err(|e| anyhow!(e))
    }?;

    post.meta.number = res_body["number"].as_u64();
    post.save().map_err(|e| anyhow!(e))?;

    println!(
        "ok: {} to {}",
        post.fullname.to_string_lossy(),
        res_body["url"].as_str().unwrap()
    );
    Ok(())
}

fn fetch(command: FetchCommand, user_config: UserConfig) -> Result<()> {
    if command.name.is_some() {
        fetch_name(command.name.unwrap(), user_config)
    } else if command.number.is_some() {
        fetch_number(command.number.unwrap(), user_config)
    } else {
        fetch_all(user_config)
    }
}

fn fetch_name(post_name: PathBuf, user_config: UserConfig) -> Result<()> {
    Post::is_invalid_name(&post_name).map_err(|e| anyhow!(e))?;

    let mut q: Vec<String> = vec![];
    q.push(format!("user:{}", user_config.user));

    if let Some(parent) = post_name.parent() {
        let parent = parent.to_string_lossy();
        if parent != "" {
            q.push(format!("category:{}", parent));
        }
    }

    if let Some(file_name) = post_name.file_name() {
        let file_name = file_name.to_string_lossy();
        if file_name != "" {
            q.push(format!("name:{}", file_name));
        }
    }
    let q = FetchParameter::Qstring(q);

    let esa_client = EsaClient::new(user_config);
    let response_json = esa_client.fetch_post(q).map_err(|e| anyhow!(e))?;

    for response_post in response_json["posts"].as_array().unwrap() {
        save_feched_to_post(response_post)?;
    }

    Ok(())
}

fn fetch_number(number: u64, user_config: UserConfig) -> Result<()> {
    let esa_client = EsaClient::new(user_config);

    let response_json = esa_client
        .fetch_post(FetchParameter::PathNumber(number))
        .map_err(|e| anyhow!(e))?;

    save_feched_to_post(&response_json)
}

fn fetch_all(user_config: UserConfig) -> Result<()> {
    let q = FetchParameter::Qstring(vec![format!("user:{}", user_config.user)]);
    let esa_client = EsaClient::new(user_config);
    let response_json = esa_client.fetch_post(q).map_err(|e| anyhow!(e))?;

    for response_post in response_json["posts"].as_array().unwrap() {
        save_feched_to_post(response_post)?;
    }

    Ok(())
}

fn save_feched_to_post(json: &Value) -> Result<()> {
    let fullname = PathBuf::from(format!(
        "{}/{}",
        json["category"].as_str().unwrap(),
        json["name"].as_str().unwrap()
    ));

    let post = Post::from_json(fullname, json);

    post.save().map_err(|e| anyhow!(e))?;

    println!(
        "ok: {} to {}",
        json["url"].as_str().unwrap(),
        post.fullname.to_string_lossy()
    );

    Ok(())
}

pub fn attach(image_file: PathBuf, user_config: UserConfig) -> Result<()> {
    let esa_client = EsaClient::new(user_config);
    let output = esa_client
        .upload_image(image_file)
        .map_err(|e| anyhow!(e))?;

    println!("{}", output);

    Ok(())
}
