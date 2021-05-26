use log::LevelFilter;
use log4rs::{
  append::{
    console::ConsoleAppender,
    rolling_file::{
      policy::compound::{
        roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
      },
      RollingFileAppender,
    },
  },
  config::{Appender, Config, Root},
  encode::json::JsonEncoder,
  filter::threshold::ThresholdFilter,
};

use crate::settings::APP_SETTINGS;

// lazy_static! {
// 	pub static ref LOGGER: Logger = init_loggers();
// }

pub fn init_logger() {
  let console_appender = ConsoleAppender::builder().build();

  let size_trigger = SizeTrigger::new(u64::pow(2, 16));

  let pattern = format!("{}.{{}}.gz", &APP_SETTINGS.logger.path);

  let fixed_roller = FixedWindowRoller::builder()
    .build(&pattern, 2)
    .expect("Could not create fixed_roller");

  let rolling_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_roller));

  let file_appender = RollingFileAppender::builder()
    .encoder(Box::new(JsonEncoder::new()))
    .build(&APP_SETTINGS.logger.path, Box::new(rolling_policy))
    .expect("Could not create file_appender");

  let config = Config::builder()
    .appender(Appender::builder().build("console_appender", Box::new(console_appender)))
    .appender(
      Appender::builder()
        .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
        .build("file_appender", Box::new(file_appender)),
    )
    .build(
      Root::builder()
        .appender("console_appender")
        .appender("file_appender")
        .build(APP_SETTINGS.logger.level.parse().unwrap_or(LevelFilter::Warn)),
    )
    .unwrap();

  log4rs::init_config(config).unwrap();
}
