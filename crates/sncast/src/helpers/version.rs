#[macro_export]
macro_rules! generate_version_parser {
    ($enum_name:ident, $variant1:ident, $variant2:ident ) => {
        use shared::print::print_as_warning;
        use anyhow::Error;

        impl std::str::FromStr for $enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let var1 = stringify!($variant1).to_lowercase();
                let var2 = stringify!($variant2).to_lowercase();

                match s.to_lowercase().as_str() {
                    v if v == var1 => Ok($enum_name::$variant1),
                    v if v == var2 => Ok($enum_name::$variant2),
                    _ => Err(format!(
                        "Invalid value '{}'. Possible values: {}, {}",
                        s, var1, var2
                    )),
                }
            }
        }

        pub fn parse_version(s: &str) -> Result<$enum_name, String> {
            let deprecation_message = "The '--version' flag is deprecated and will be removed in the future. Version 3 transactions will become the default and only available version.";
            print_as_warning(&Error::msg(deprecation_message));

            let parsed_enum = s.parse::<$enum_name>()?;

            println!("Parsed enum: {:?}", parsed_enum);

            Ok(parsed_enum)
        }
    };
}
