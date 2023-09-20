use dotenv::dotenv;

mod e2e;
mod integration;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    dotenv().ok().unwrap();
}
