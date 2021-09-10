use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Component, PathBuf};

use super::error::Error;

pub fn is_post_name_valid(post_name: &PathBuf) -> Result<(), Error> {
    let valid = post_name.components().filter(|x| {
        if let Component::Normal(_) = x {
            true
        } else {
            false
        }
    });
    let invalid = post_name.components().filter(|x| {
        if let Component::Normal(_) = x {
            false
        } else {
            true
        }
    });
    if valid.count() <= 1 || 1 <= invalid.count() {
        return Err(Error::PostNameIsInvalid);
    }

    Ok(())
}

pub fn save_post(
    post_name: &PathBuf,
    body_md_content: &[u8],
    settings_json_content: &[u8],
) -> Result<(), Error> {
    if let Err(error) = create_dir_all(post_name) {
        return Err(Error::SavePost(error));
    }

    let mut body_md = post_name.clone();
    body_md.push("body.md");

    let result = apply_content(&body_md, body_md_content);
    if result.is_err() {
        return result;
    }

    let mut settings_json = post_name.clone();
    settings_json.push("settings.json");

    let result = apply_content(&settings_json, settings_json_content);
    if result.is_err() {
        return result;
    }

    Ok(())
}

pub fn apply_content(file: &PathBuf, content: &[u8]) -> Result<(), Error> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file);

    if let Err(error) = file {
        return Err(Error::SavePost(error));
    };

    let mut f = BufWriter::new(file.unwrap());

    if let Err(error) = f.write(content) {
        return Err(Error::SavePost(error));
    };

    Ok(())
}
