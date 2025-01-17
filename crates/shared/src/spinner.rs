use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::time::Duration;

/// Styled spinner that uses [`ProgressBar`].
/// Automatically finishes and clears itself when dropped.
pub struct Spinner(ProgressBar);
impl Spinner {
    /// Create [`Spinner`] with a message.
    pub fn create_with_message(message: impl Into<Cow<'static, str>>) -> Self {
        let spinner = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("\n{spinner} {msg}\n")
            .expect("template is static str and should be valid");
        spinner.set_style(style);
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(message);
        Self(spinner)
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.0.finish_and_clear();
    }
}
