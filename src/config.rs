// use std::collections::BTreeMap;
use std::io::prelude::*;
use std::fs::File;

use clap::{Arg, App};
use log::LogLevel;
use toml;

#[cfg(test)]
mod tests {
    #[test]
    fn it_parses_yaml_into_config() {
        let config_input = r#"
[vault]
hosts = ["http://localhost:8200"]
token = "test12345"
        "#;

        let config = super::parsed_config(config_input);
        let vault = config.vault.unwrap();
        assert_eq!(vault.hosts, vec!["http://localhost:8200".to_string()])
    }

    #[test]
    fn it_ignores_empty_entries_in_hosts() {
        let config_input = r#"
[vault]
hosts = ["http://localhost:8200", ""]
token = "test12345"
        "#;

        let config = super::parsed_config(config_input);
        let vault = config.vault.unwrap();
        assert_eq!(vault.hosts, vec!["http://localhost:8200".to_string()])
    }
}

#[derive(Debug)]
pub struct Config {
    pub vault: Option<VaultConfig>,
    pub log_level: LogLevel,
}

#[derive(Debug)]
pub struct VaultConfig {
    pub hosts: Vec<String>,
    pub token: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            vault: None,
            log_level: LogLevel::Warn,
        }
    }
}

pub fn get_config() -> Config {
    let matches = App::new("nebulosus")
        .version(crate_version!())
        .arg(Arg::with_name("debug")
                           .short("d")
                           .multiple(true)
                           .help("Sets the level of debugging information"))
        .arg(Arg::with_name("CONFIG")
                           .short("c")
                           .long("config")
                           .help("Sets a custom config file")
                           .takes_value(true))
        .get_matches();

    let log_level = match matches.occurrences_of("debug") {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        3 | _ => LogLevel::Trace,
    };

    let config_file = matches.value_of("CONFIG").unwrap_or("/etc/nebulosus/nebulosus.conf");
    let mut s = String::new();
    match File::open(config_file) {
        Err(e) => {
            warn!("Error opening config file ({:?}, using defaults", e);
            return Config::default();
        }
        Ok(mut f) => {
            match f.read_to_string(&mut s) {
                Err(e) => {
                    warn!("Error opening config file ({:?}, using defaults", e);
                    return Config::default();
                },
                _ => {}
            }
        }
    };

    let mut config = parsed_config(&s[..]);

    config.log_level = log_level;
    config
}

fn parsed_config(toml: &str) -> Config {

    let parsed = match toml::Parser::new(toml).parse() {
        Some(value) => value,
        None => panic!("Bad config format"),
    };
    trace!("parsed == {:?}", parsed);

    Config {
        vault: parse_vault_config(&parsed["vault"]),
        log_level: LogLevel::Warn
    }
}

fn parse_vault_config(toml: &toml::Value) -> Option<VaultConfig> {
    // let toml = parsed.clone();
    let hosts: toml::Value = match toml.lookup("hosts") {
        Some(s) => s.to_owned(),
        None => return None,
    };

    let hosts: Vec<String> = match hosts {
         toml::Value::Array(ref s) => s.iter().map(|s| match s.as_str() {
            Some(s) => s.to_string(),
            None => String::new(),
            }).filter(|s| &s[..] != "").collect(),
        _ => vec![],
    };

    debug!("Hosts are: {:?}", hosts);

    let token: toml::Value = match toml.lookup("token") {
        Some(s) => s.to_owned(),
        None => return None,
    };
    let token: String = match token {
        toml::Value::String(ref s) => s.to_string(),
        _ => "".to_string(),
    };

    Some(VaultConfig {
        hosts: hosts,
        token: token,
    })
}
