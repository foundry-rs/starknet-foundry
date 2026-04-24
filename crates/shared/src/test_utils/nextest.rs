#[must_use]
pub fn is_nextest() -> bool {
    std::env::var("NEXTEST").as_deref() == Ok("1")
}
