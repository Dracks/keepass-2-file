use std::sync::LazyLock;

use colored::Colorize;

pub static LOG_PREFIX: LazyLock<LogPrefix> = LazyLock::new(LogPrefix::new);

#[derive(Clone)]
pub struct LogPrefix {
    pub error: String,
}

impl LogPrefix {
    fn new() -> Self {
        LogPrefix {
            error: "error".red().to_string(),
        }
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
        let _colorized = OverrideColorize::new(true);
        let subject = LogPrefix::new();

        assert_eq!(subject.error, "\u{1b}[31merror\u{1b}[0m")
    }

    #[test]
    fn test_log_prefix_not_colorized() {
        let _colorized = OverrideColorize::new(false);
        let subject = LogPrefix::new();

        assert_eq!(subject.error, "error")
    }
}
