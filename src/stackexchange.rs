use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use crate::config::{project_dir, Config};
use crate::error::{Error, ErrorKind, Result};

/// StackExchange API v2.2 URL
const SE_URL: &str = "http://api.stackexchange.com/2.2/";

/// Filter generated to include only the fields needed to populate
/// the structs below. Go here to make new filters:
/// [create filter](https://api.stackexchange.com/docs/create-filter).
const SE_FILTER: &str = ".DND5X2VHHUH8HyJzpjo)5NvdHI3w6auG";

/// Pagesize when fetching all SE sites. Should be good for many years...
const SE_SITES_PAGESIZE: u16 = 10000;

/// This structure allows interacting with parts of the StackExchange
/// API, using the `Config` struct to determine certain API settings and options.
pub struct StackExchange {
    client: Client,
    config: Config,
}

/// This structure allows interacting with locally cached StackExchange metadata.
pub struct LocalStorage {
    sites: Option<Vec<Site>>,
    filename: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Site {
    pub api_site_parameter: String,
    pub site_url: String,
}

/// Represents a StackExchange answer with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/answer)
#[derive(Deserialize, Debug)]
pub struct Answer {
    #[serde(rename = "answer_id")]
    pub id: u32,
    pub score: i32,
    #[serde(rename = "body_markdown")]
    pub body: String,
    pub is_accepted: bool,
}

/// Represents a StackExchange question with a custom selection of fields from
/// the [StackExchange docs](https://api.stackexchange.com/docs/types/question)
#[derive(Deserialize, Debug)]
pub struct Question {
    #[serde(rename = "question_id")]
    pub id: u32,
    pub score: i32,
    pub answers: Vec<Answer>,
    pub title: String,
    #[serde(rename = "body_markdown")]
    pub body: String,
}

/// Internal struct that represents the boilerplate response wrapper from SE API.
#[derive(Deserialize, Debug)]
struct ResponseWrapper<T> {
    items: Vec<T>,
}

impl StackExchange {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        StackExchange { client, config }
    }

    /// Search against the search/advanced endpoint with a given query.
    /// Only fetches questions that have at least one answer.
    /// TODO async
    /// TODO parallel requests over multiple sites
    pub fn search(&self, q: &str) -> Result<Vec<Question>> {
        let resp_body = self
            .client
            .get(stackexchange_url("search/advanced"))
            .header("Accepts", "application/json")
            .query(&self.get_default_opts())
            .query(&[
                ("q", q),
                ("pagesize", &self.config.limit.to_string()),
                ("page", "1"),
                ("answers", "1"),
                ("order", "desc"),
                ("sort", "relevance"),
            ])
            .send()
            .map_err(|e| {
                // TODO explore legit errors such as not connected to internet
                Error::se(format!(
                    "Error encountered while querying StackExchange: {}",
                    e
                ))
            })?;

        let gz = GzDecoder::new(resp_body);
        let wrapper: ResponseWrapper<Question> = serde_json::from_reader(gz).map_err(|e| {
            Error::se(format!(
                "Error decoding questions from the StackExchange API: {}",
                e
            ))
        })?;
        let qs = wrapper
            .items
            .into_iter()
            .map(|mut q| {
                q.answers.sort_unstable_by_key(|a| -a.score);
                q
            })
            .collect();
        Ok(qs)
    }

    fn get_default_opts(&self) -> HashMap<&str, &str> {
        let mut params = HashMap::new();
        params.insert("site", self.config.site.as_str());
        params.insert("filter", &SE_FILTER);
        if let Some(key) = &self.config.api_key {
            params.insert("key", key.as_str());
        }
        params
    }
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let project = project_dir()?;
        let dir = project.cache_dir();
        fs::create_dir_all(&dir).map_err(|_| Error::create_dir(&dir.to_path_buf()))?;
        Ok(LocalStorage {
            sites: None,
            filename: dir.join("sites.json"),
        })
    }

    // TODO make this async, inform user if we are downloading
    // TODO issue EmptySites from here when appropriate
    pub fn sites(&mut self) -> Result<&Vec<Site>> {
        // Stop once Option ~ Some or Result ~ Err
        if self.sites.is_some() {
            return Ok(self.sites.as_ref().unwrap()); // safe
        }
        if self.fetch_local_sites()?.is_some() {
            return Ok(self.sites.as_ref().unwrap()); // safe
        }
        self.fetch_remote_sites()?;
        self.sites
            .as_ref()
            .ok_or_else(|| Error::from("Code failure in site listing retrieval"))
    }

    pub fn update_sites(&mut self) -> Result<()> {
        self.fetch_remote_sites()
    }

    pub fn validate_site(&mut self, site_code: &str) -> Result<bool> {
        let sites = self.sites()?;
        if sites.is_empty() {
            return Err(Error {
                kind: ErrorKind::EmptySites,
                error: String::from(""),
            });
        }
        Ok(sites
            .iter()
            .any(|site| site.api_site_parameter == *site_code))
    }

    fn fetch_local_sites(&mut self) -> Result<Option<()>> {
        if let Ok(file) = File::open(&self.filename) {
            self.sites =
                serde_json::from_reader(file).map_err(|_| Error::malformed(&self.filename))?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    // TODO decide whether or not I should give LocalStorage an api key..
    // TODO cool loading animation?
    fn fetch_remote_sites(&mut self) -> Result<()> {
        let resp_body = Client::new()
            .get(stackexchange_url("sites"))
            .header("Accepts", "application/json")
            .query(&[
                ("pagesize", SE_SITES_PAGESIZE.to_string()),
                ("page", "1".to_string()),
            ])
            .send()
            .map_err(|e| {
                Error::se(format!(
                    "Error requesting sites from StackExchange API: {}",
                    e
                ))
            })?;
        let gz = GzDecoder::new(resp_body);
        let wrapper: ResponseWrapper<Site> = serde_json::from_reader(gz).map_err(|e| {
            Error::se(format!(
                "Error decoding sites from the StackExchange API: {}",
                e
            ))
        })?;
        self.sites = Some(wrapper.items);
        self.store_local_sites()
    }

    fn store_local_sites(&self) -> Result<()> {
        let file = File::create(&self.filename).map_err(|_| Error::create_file(&self.filename))?;
        serde_json::to_writer(file, &self.sites).map_err(|_| Error::write_file(&self.filename))
    }
}

/// Creates url from const string; can technically panic
fn stackexchange_url(path: &str) -> Url {
    let mut url = Url::parse(SE_URL).unwrap();
    url.set_path(path);
    url
}

#[cfg(test)]
mod tests {
    // TODO for both, detect situation and print helpful error message
    #[test]
    fn test_invalid_api_key() {
        assert!(true)
    }
    #[test]
    fn test_invalid_filter() {
        assert!(true)
    }
}
