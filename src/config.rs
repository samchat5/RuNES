use config::Config;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG: Config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();
}
