use regex::Regex;

pub fn extract_string_from_flag(cmdline: &str, flag: &str) -> Option<String> {
    let pattern = format!(r"{}\s*=?\s*(\S+)", regex::escape(flag));
    let re = Regex::new(&pattern).ok()?;

    re.captures(cmdline)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn extract_port_from_flag(cmdline: &str, flag: &str) -> Option<u16> {
    extract_string_from_flag(cmdline, flag).and_then(|port_str| port_str.parse().ok())
}
