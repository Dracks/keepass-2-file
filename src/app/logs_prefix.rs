use std::sync::LazyLock;

use colored::Colorize;

pub static LOG_PREFIX: LazyLock<LogPrefix> = LazyLock::new(|| LogPrefix::new(true));

#[derive(Clone)]
pub struct LogPrefix {
    pub error: String,
    pub warning: String,
}

impl LogPrefix {
    pub fn new(color: bool) -> Self {
        let error = if color {
            "error".red().to_string()
        } else {
            "error".to_string()
        };

        let warning = if color {
            "warning".yellow().to_string()
        } else {
            "warning".to_string()
        };

        LogPrefix { error, warning }
    }
}

#[cfg(test)]
pub mod test {
    use colored::control;

    use super::*;

    pub struct OverrideColorize {}

    impl OverrideColorize {
        pub fn new(color: bool) -> Self {
            control::SHOULD_COLORIZE.set_override(color);
            Self {}
        }
    }

    impl Drop for OverrideColorize {
        fn drop(&mut self) {
            control::SHOULD_COLORIZE.unset_override();
        }
    }

    #[test]
    fn test_log_prefix_colorized() {
        control::set_override(true);
        let subject = LogPrefix::new(true);

        assert_eq!(subject.warning, "\u{1b}[33mwarning\u{1b}[0m");
        assert_eq!(subject.error, "\u{1b}[31merror\u{1b}[0m")
    }

    #[test]
    fn test_log_prefix_not_colorized() {
        let subject = LogPrefix::new(false);

        assert_eq!(subject.error, "error");
        assert_eq!(subject.warning, "warning");
    }
}
