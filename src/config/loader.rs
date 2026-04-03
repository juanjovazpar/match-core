use crate::config::types::Config;
use config::{Config as RawConfig, Environment, File};

pub fn load_config() -> Config {
    let builder = RawConfig::builder()
        .add_source(File::with_name("config.yaml").required(true))
        .add_source(Environment::default().separator("__"));

    let raw = builder.build().expect("Failed to build config");

    raw.try_deserialize::<Config>()
        .expect("Failed to deserialize config")
}
