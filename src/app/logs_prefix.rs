use std::sync::LazyLock;

use colored::{control, Colorize};

pub static LOG_PREFIX: LazyLock<LogPrefix> = LazyLock::new(LogPrefix::new);

#[derive(Clone)]
pub struct LogPrefix {
    pub error: String,
}

impl LogPrefix {
    fn build(enable_colorize: bool) -> Self {
        if enable_colorize {
            LogPrefix {
                error: "error".red().to_string(),
            }
        } else {
            LogPrefix {
                error: "error".to_string(),
            }
        }
    }
    fn new() -> Self {
        let enable_colorize = control::SHOULD_COLORIZE.should_colorize();
        Self::build(enable_colorize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_log_prefix_colorized() {
        let subject = LogPrefix::build(true);

        assert_eq!(subject.error, "\u{1b}[31merror\u{1b}[0m")
    }

    #[test]
    fn test_log_prefix_not_colorized() {
        let subject = LogPrefix::build(false);

        assert_eq!(subject.error, "error")
    }
}
