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
    #[must_use]
    pub fn get_bool(prop: &str, default: bool) -> bool {
        CONF.get_bool(prop).unwrap_or(default)
    }

    #[must_use]
    pub fn get_string_with_default(prop: &str, default: &str) -> String {
        CONF.get_string(prop)
            .unwrap_or_else(|_| default.to_string())
    }

    pub fn get_string(prop: &str) -> Option<String> {
        CONF.get_string(prop).ok()
    }

    pub fn get_int<T: Into<i64> + From<i64>>(prop: &str, default: T) -> T {
        CONF.get_int(prop).unwrap_or_else(|_| default.into()).into()
    }
}
