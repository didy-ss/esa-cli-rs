use structopt::StructOpt;

use core::panic;

use serde_json::{json, to_string_pretty};

use std::fs::{metadata, read_to_string};
use std::path::PathBuf;
use std::process::Command;

mod libs;
use libs::client::Client;
use libs::esa_info::EsaInfo;
use libs::opt_args::{GetOpt, Opts, SubOpts};
use libs::settings_json::SettingsJson;
use libs::util::{apply_content, is_post_name_valid, save_post};

fn main() {
    let opts: Opts = Opts::from_args();

    let esa_info = match EsaInfo::from_file() {
        Ok(info) => info,
        Err(error) => panic!("{}", error),
    };

    match opts.sub {
        SubOpts::New { post_name } => new(post_name),
        SubOpts::Post { post_name } => post(post_name, esa_info),
        SubOpts::Attach { file_path } => attach(file_path, esa_info),
        SubOpts::Get(get_opt) => get(get_opt, esa_info),
    };
}

fn new(post_name: PathBuf) {
    if let Err(error) = is_post_name_valid(&post_name) {
        panic!("{}", error);
    }

    if post_name.exists() {
        panic!("{} exists", post_name.to_string_lossy());
    }

    let settings_content = r#"{
  "tags": [],
  "wip": true,
  "number": null
}"#
    .as_bytes();

    if let Err(error) = save_post(&post_name, "".as_bytes(), settings_content) {
        panic!("{}", error);
    }

    println!("ok: {}", post_name.to_string_lossy());
}

fn post(post_name: PathBuf, esa_info: EsaInfo) {
    if !post_name.exists() {
        panic!("{} does not exist", post_name.to_string_lossy());
    }

    let mut body_md = post_name.clone();
    body_md.push("body.md");
    let body = read_to_string(body_md).unwrap();

    let mut settings_json = post_name.clone();
    settings_json.push("settings.json");
    let mut settings = SettingsJson::from_file(&settings_json);

    let esa_client = Client::new(esa_info.api_key);
    let params = json!({
        "post": {
            "name": post_name.file_name().unwrap().to_string_lossy(),
            "body_md": body,
            "tags": settings.tags,
            "category": post_name.parent().unwrap().to_string_lossy(),
            "wip": settings.wip
        }
    });

    let res_body = if settings.number.is_none() {
        esa_client
            .post(format!("/v1/teams/{}/posts", esa_info.team), params)
            .unwrap()
    } else {
        esa_client
            .patch(
                format!(
                    "/v1/teams/{}/posts/{}",
                    esa_info.team,
                    settings.number.unwrap()
                ),
                params,
            )
            .unwrap()
    };

    settings.number = res_body["number"].as_u64();
    if let Err(error) = apply_content(
        &settings_json,
        to_string_pretty(&settings).unwrap().as_bytes(),
    ) {
        panic!("{}", error);
    }

    println!("ok: {} to {}", post_name.to_string_lossy(), res_body["url"].as_str().unwrap());
}

fn attach(file_path: PathBuf, esa_info: EsaInfo) {
    let esa_client = Client::new(esa_info.api_key);
    let mine = mime_guess::from_path(file_path.clone()).first().unwrap();
    let params = json!({
        "type": mine.to_string(),
        "size": metadata(file_path.clone()).unwrap().len(),
        "name": file_path.to_string_lossy(),
    });
    let policies = esa_client
        .post(
            format!("/v1/teams/{}/attachments/policies", esa_info.team),
            params,
        )
        .unwrap();

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
            &format!("file=@{}", file_path.to_string_lossy()),
            "--include",
            policies["attachment"]["endpoint"].as_str().unwrap(),
        ])
        .output()
        .expect("Failed to Execute");

    println!("{}", String::from_utf8(output.stdout).unwrap());
}

fn get(opt: GetOpt, esa_info: EsaInfo) {
    if opt.name.is_some() {
        get_name(opt.name.unwrap(), esa_info);
    } else if opt.number.is_some() {
        get_number(opt.number.unwrap(), esa_info);
    } else {
        get_all(esa_info);
    }
}

fn get_name(post_name: PathBuf, esa_info: EsaInfo) {
    if let Err(error) = is_post_name_valid(&post_name) {
        panic!("{}", error);
    }

    let mut q: Vec<String> = vec![];
    q.push(format!("user:{}", esa_info.user));

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
    let q_str = q.join(" ");

    let esa_client = Client::new(esa_info.api_key);
    let res_body = esa_client
        .get(format!("/v1/teams/{}/posts?q={}", esa_info.team, q_str))
        .unwrap();
    for post in res_body["posts"].as_array().unwrap() {
        let settings = SettingsJson::from_json(post);

        let name = PathBuf::from(format!(
            "{}/{}",
            post["category"].as_str().unwrap(),
            post["name"].as_str().unwrap()
        ));
        if let Err(error) = save_post(
            &name,
            post["body_md"].as_str().unwrap().as_bytes(),
            to_string_pretty(&settings).unwrap().as_bytes(),
        ) {
            panic!("{}", error);
        }
        println!("ok: {} to {}", post["url"].as_str().unwrap(), name.to_string_lossy());
    }
}

fn get_number(number: u64, esa_info: EsaInfo) {
    let esa_client = Client::new(esa_info.api_key);
    let res_body = esa_client
        .get(format!("/v1/teams/{}/posts/{}", esa_info.team, number))
        .unwrap();

    let settings = SettingsJson::from_json(&res_body);

    let name = PathBuf::from(format!(
        "{}/{}",
        res_body["category"].as_str().unwrap(),
        res_body["name"].as_str().unwrap()
    ));
    if let Err(error) = save_post(
        &name,
        res_body["body_md"].as_str().unwrap().as_bytes(),
        to_string_pretty(&settings).unwrap().as_bytes(),
    ) {
        panic!("{}", error);
    }

    println!("ok: {} to {}", res_body["url"].as_str().unwrap(), name.to_string_lossy());
}

fn get_all(esa_info: EsaInfo) {
    let esa_client = Client::new(esa_info.api_key);
    let res_body = esa_client
        .get(format!(
            "/v1/teams/{}/posts?q=user:{}",
            esa_info.team, esa_info.user
        ))
        .unwrap();

    for post in res_body["posts"].as_array().unwrap() {
        let settings = SettingsJson::from_json(post);

        if post["category"].is_null() {
            continue;
        }

        let name = PathBuf::from(format!(
            "{}/{}",
            post["category"].as_str().unwrap(),
            post["name"].as_str().unwrap()
        ));
        if let Err(error) = save_post(
            &name,
            post["body_md"].as_str().unwrap().as_bytes(),
            to_string_pretty(&settings).unwrap().as_bytes(),
        ) {
            panic!("{}", error);
        }

        println!("ok: {} to {}", post["url"].as_str().unwrap(), name.to_string_lossy());
    }
}
