use crate::base::config::Config;
use crate::base::result::Result;

pub(crate) struct Logger;

impl Logger {
    pub(crate) fn init(config: &Config) -> Result<()> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(config.log_level.to_level_filter())
            .chain(std::io::stdout())
            .apply()?;

        Ok(())
    }
}
