use clap::{App, AppSettings, Arg};

use crate::config;
use crate::config::Config;
use crate::error::Result;

// TODO --sites plural
// TODO --add-site (in addition to defaults)
pub struct Opts {
    pub list_sites: bool,
    pub update_sites: bool,
    pub set_api_key: Option<String>,
    pub query: Option<String>,
    pub config: Config,
}

pub fn get_opts() -> Result<Opts> {
    let config = config::user_config()?;
    let limit = &config.limit.to_string();
    let sites = &config.sites.join(";");
    let matches = App::new("so")
        .setting(AppSettings::ColoredHelp)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("list-sites")
                .long("list-sites")
                .help("Print available StackExchange sites"),
        )
        .arg(
            Arg::with_name("update-sites")
                .long("update-sites")
                .help("Update cache of StackExchange sites"),
        )
        .arg(
            Arg::with_name("set-api-key")
                .long("set-api-key")
                .number_of_values(1)
                .takes_value(true)
                .help("Set StackExchange API key"),
        )
        .arg(
            Arg::with_name("site")
                .long("site")
                .short("s")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .default_value(sites)
                .help("StackExchange site code to search"),
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .short("l")
                .number_of_values(1)
                .takes_value(true)
                .default_value(limit)
                .validator(|s| s.parse::<u32>().map_err(|e| e.to_string()).map(|_| ()))
                .help("Question limit"),
        )
        .arg(
            Arg::with_name("lucky")
                .long("lucky")
                .help("Print the top-voted answer of the most relevant question"),
        )
        .arg(
            Arg::with_name("no-lucky")
                .long("no-lucky")
                .help("Disable lucky")
                .conflicts_with("lucky")
                .hidden(!config.lucky),
        )
        .arg(
            Arg::with_name("query")
                .multiple(true)
                .index(1)
                .required_unless_one(&["list-sites", "update-sites", "set-api-key"]),
        )
        .arg(
            Arg::with_name("duckduckgo")
                .long("duckduckgo")
                .help("Use DuckDuckGo as a search engine"),
        )
        .arg(
            Arg::with_name("no-duckduckgo")
                .long("no-duckduckgo")
                .help("Disable duckduckgo")
                .conflicts_with("duckduckgo")
                .hidden(!config.duckduckgo),
        )
        .get_matches();
    let lucky = match (matches.is_present("lucky"), matches.is_present("no-lucky")) {
        (true, _) => true,
        (_, true) => false,
        _ => config.lucky,
    };
    let duckduckgo = match (
        matches.is_present("duckduckgo"),
        matches.is_present("no-duckduckgo"),
    ) {
        (true, _) => true,
        (_, true) => false,
        _ => config.duckduckgo,
    };
    Ok(Opts {
        list_sites: matches.is_present("list-sites"),
        update_sites: matches.is_present("update-sites"),
        set_api_key: matches.value_of("set-api-key").map(String::from),
        query: matches
            .values_of("query")
            .map(|q| q.collect::<Vec<_>>().join(" ")),
        config: Config {
            // these unwraps are safe via clap default values & validators
            limit: matches.value_of("limit").unwrap().parse::<u16>().unwrap(),
            sites: matches
                .values_of("site")
                .unwrap()
                .map(|s| s.split(';'))
                .flatten()
                .map(String::from)
                .collect(),
            api_key: matches
                .value_of("set-api-key")
                .map(String::from)
                .or(config.api_key),
            lucky,
            duckduckgo,
        },
    })
}

// TODO how can I test this App given https://users.rust-lang.org/t/help-with-idiomatic-rust-and-ownership-semantics/43880
// Maybe pass get_opts a closure that produces the Config...
