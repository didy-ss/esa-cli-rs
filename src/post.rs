use regex::Regex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fmt;
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io;
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct Post {
    pub fullname: PathBuf,
    body: Option<String>,
    pub meta: Meta,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Meta {
    tags: Vec<String>,
    wip: bool,
    pub number: Option<u64>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("file name must be ([category/]+)filename")]
    NameInvalid,

    #[error("`{0}` already exists")]
    PostExists(String),

    #[error("`{0}` does not exist")]
    PostNotExists(String),

    #[error("failed to write directory or file: {0}")]
    PostNameWriteFailed(#[from] io::Error),

    #[error("Invalid Format")]
    PostFormatInvalid,

    #[error("meta info is invalid: {0}")]
    MetaTomlInvalid(#[from] toml::de::Error),
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

impl Post {
    pub fn is_invalid_name(fullname: &Path) -> Result<(), Error> {
        let count = fullname.components().count();
        let invalid = count <= 1
            || fullname
                .components()
                .filter(|x| matches!(x, Component::Normal(_)))
                .count()
                < count;

        if invalid {
            return Err(Error::NameInvalid);
        }

        Ok(())
    }

    pub fn new(fullname: PathBuf) -> Result<Post, Error> {
        Post::is_invalid_name(&fullname)?;

        if fullname.exists() {
            return Err(Error::PostExists(String::from(fullname.to_string_lossy())));
        }

        Ok(Post {
            fullname,
            body: None,
            meta: Meta {
                tags: vec![],
                wip: true,
                number: None,
            },
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        let mut file_path = self.fullname.clone();
        file_path.set_extension("md");
        let parent = file_path.parent().unwrap();
        create_dir_all(parent).map_err(Error::PostNameWriteFailed)?;

        let content = format!(
            r#"+++
{}+++

{}
"#,
            self.meta,
            self.body.clone().unwrap_or_else(|| "".to_string()),
        );

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file_path)
            .map_err(Error::PostNameWriteFailed)?;

        let mut f = BufWriter::new(file);
        f.write(content.as_bytes())
            .map_err(Error::PostNameWriteFailed)?;

        Ok(())
    }

    pub fn from_file(fullname: PathBuf) -> Result<Post, Error> {
        if !fullname.exists() {
            return Err(Error::PostNotExists(String::from(
                fullname.to_string_lossy(),
            )));
        }

        let content = read_to_string(&fullname).unwrap();

        let mut fullname = fullname;
        fullname.set_extension("");
        Post::from_str(fullname, content)
    }

    fn from_str(fullname: PathBuf, content: String) -> Result<Post, Error> {
        let re = Regex::new(
            r"^[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))"
        ).unwrap();

        let caps = re.captures(&content).ok_or(Error::PostFormatInvalid)?;

        let (toml, body) = {
            let meta = caps.get(1).unwrap().as_str();
            let body = caps.get(2).map(|m| m.as_str().to_string());
            (meta, body)
        };

        let meta: Meta = toml::from_str(toml).map_err(Error::MetaTomlInvalid)?;
        Ok(Post {
            fullname,
            body,
            meta,
        })
    }

    pub fn from_json(fullname: PathBuf, json: &Value) -> Post {
        return Post {
            fullname,
            body: json["body_md"].as_str().map(|x| x.to_string()),
            meta: Meta {
                tags: json["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
                wip: json["wip"].as_bool().unwrap(),
                number: json["number"].as_u64(),
            },
        };
    }

    pub fn to_json(&self) -> Value {
        json!(
            {
                "post": {
                    "category": self.fullname.parent().unwrap().to_string_lossy(),
                    "name": self.fullname.file_name().unwrap().to_string_lossy(),
                    "body_md": self.body.clone().unwrap_or_else(|| "".to_string()),
                    "tags": self.meta.tags,
                    "wip": self.meta.wip,
                }
            }
        )
    }
}
