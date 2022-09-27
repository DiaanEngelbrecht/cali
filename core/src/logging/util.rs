use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

pub fn setup() {
    let log_pattern = "{d(%Y-%m-%dT%H:%M:%S)} [{l}] {f}:{L} {m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("tonic", LevelFilter::Info))
        .logger(Logger::builder().build("sqlx", LevelFilter::Error))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}
