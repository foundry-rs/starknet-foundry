use std::borrow::Cow;

pub trait OutputTransform: Clone + Default {
    fn transform_stdout(stdout: &[u8]) -> Cow<'_, [u8]>;
    fn transform_stderr(stderr: &[u8]) -> Cow<'_, [u8]>;
}

#[derive(Clone, Default)]
pub struct PassByPrint;

impl OutputTransform for PassByPrint {
    fn transform_stdout(stdout: &[u8]) -> Cow<'_, [u8]> {
        stdout.into()
    }
    fn transform_stderr(stderr: &[u8]) -> Cow<'_, [u8]> {
        stderr.into()
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputTransform, PassByPrint};
    use std::borrow::Cow;

    #[test]
    fn is_equal() {
        let message = b"asdfghjkgfdsertyh";

        let stderr = PassByPrint::transform_stderr(message);
        let stdout = PassByPrint::transform_stdout(message);

        assert_eq!(stderr, Cow::Borrowed(message));
        assert_eq!(stdout, Cow::Borrowed(message));
    }
}
