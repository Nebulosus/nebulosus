#[macro_use] extern crate clap;
extern crate hashicorp_vault;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate toml;

use hashicorp_vault as vault;

mod config;

use config::{get_config, Config};

fn main() {
    let config: Config = get_config();
    simple_logger::init_with_level(config.log_level).unwrap();

    if config.vault.is_none() {
        panic!("Nebulosus requires vault to run!");
    }

    info!("Hello, world! Setting up Nebulosus with {:?}", config);
    let vault = config.vault.unwrap();
    let client = vault::Client::new(
        vault.hosts.iter().map(|s| &s[..]).collect(),
        &vault.token[..]
    );

}


