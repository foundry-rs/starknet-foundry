use crate::Config;
use anyhow::anyhow;
use serde_json::Number;
use std::env;

pub fn resolve_env_variables(config: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    match config {
        serde_json::Value::Object(map) => {
            let val = map
                .into_iter()
                .map(|(k, v)| -> anyhow::Result<(String, serde_json::Value)> {
                    Ok((k, resolve_env_variables(v)?))
                })
                .collect::<anyhow::Result<serde_json::Map<String, serde_json::Value>>>()?;
            Ok(serde_json::Value::Object(val))
        }
        serde_json::Value::Array(val) => {
            let val = val
                .into_iter()
                .map(resolve_env_variables)
                .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;
            Ok(serde_json::Value::Array(val))
        }
        serde_json::Value::String(val) if val.starts_with('$') => resolve_env_variable(&val),
        val => Ok(val),
    }
}

fn resolve_env_variable(var: &str) -> anyhow::Result<serde_json::Value> {
    assert!(var.starts_with('$'));
    let mut initial_value = &var[1..];
    if initial_value.starts_with('{') && initial_value.ends_with('}') {
        initial_value = &initial_value[1..initial_value.len() - 1];
    }
    let value = env::var(initial_value)?;

    if let Ok(value) = value.parse::<Number>() {
        return Ok(serde_json::Value::Number(value));
    }
    if let Ok(value) = value.parse::<bool>() {
        return Ok(serde_json::Value::Bool(value));
    }
    Ok(serde_json::Value::String(value))
}

fn get_with_ownership(config: serde_json::Value, key: &str) -> Option<serde_json::Value> {
    match config {
        serde_json::Value::Object(mut map) => map.remove(key),
        _ => None,
    }
}

fn get_profile(
    raw_config: serde_json::Value,
    tool: &str,
    profile: &str,
) -> Option<serde_json::Value> {
    let profile_name = profile;
    let tool_config = get_with_ownership(raw_config, tool)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    get_with_ownership(tool_config, profile_name)
}

pub enum Profile {
    None,
    Default,
    Some(String),
}

pub fn load_config<T: Config + Default>(
    raw_config: serde_json::Value,
    profile: Profile,
) -> anyhow::Result<T> {
    let raw_config_json = match profile {
        Profile::None => raw_config,
        Profile::Default => get_profile(raw_config, T::tool_name(), "default")
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        Profile::Some(profile) => get_profile(raw_config, T::tool_name(), &profile)
            .ok_or_else(|| anyhow!("Profile [{profile}] not found in config"))?,
    };
    T::from_raw(resolve_env_variables(raw_config_json)?)
}
