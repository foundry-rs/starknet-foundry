mod e2e;
mod integration;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    use dotenv::dotenv;
    dotenv().unwrap();
}
