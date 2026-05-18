#[cfg(test)]
#[derive(Debug, Drop, Fuzzable)]
struct Point {
    x: u64,
    y: u64,
}
