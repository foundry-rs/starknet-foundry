wit_bindgen::generate!({
    inline: r#"
        package testing:oracle;

        world oracle {
            export add: func(left: u64, right: u64) -> u64;
            export err: func() -> result<string, string>;
        }
    "#
});

struct MyOracle;

impl Guest for MyOracle {
    fn add(left: u64, right: u64) -> u64 {
        left + right
    }

    fn err() -> Result<String, String> {
        Err("failed hard".into())
    }
}

export!(MyOracle);
