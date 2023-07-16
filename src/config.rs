use config::Config as OtherConfig;
use lazy_static::lazy_static;

lazy_static! {
    static ref CONF: OtherConfig = OtherConfig::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();
}

pub struct Config {}

impl Config {
    pub fn get_bool(prop: &str, default: bool) -> bool {
        CONF.get_bool(prop).unwrap_or(default)
    }

    pub fn get_string(prop: &str, default: &str) -> String {
        CONF.get_string(prop).unwrap_or(default.to_string())
    }
}