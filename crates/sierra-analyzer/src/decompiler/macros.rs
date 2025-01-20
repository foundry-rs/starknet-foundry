/// Macro to parse the debug name from a libfunc or a function ID,
/// using the debug_name if present or falling back to the ID field
#[macro_export]
macro_rules! parse_element_name {
    ($element_id:expr) => {
        if let Some(debug_name) = &$element_id.debug_name {
            debug_name.to_string()
        } else {
            $element_id.id.to_string()
        }
    };
}

/// Macro to parse the debug name or get the name from a provided map,
/// or fallback to the ID. This is used to match the element ID with
/// its corresponding libfunc or type name
#[macro_export]
macro_rules! parse_element_name_with_fallback {
    ($element_id:expr, $fallback_map:expr) => {
        $element_id
            .debug_name
            .as_ref()
            .map(|name| name.to_string())
            .or_else(|| {
                $fallback_map
                    .get($element_id.id as usize)
                    .map(|name| name.to_string())
                    .or_else(|| Some(format!("[{}]", $element_id.id)))
            })
            .unwrap()
    };
}

/// Macro to extract parameters from the args field of a GenInvocation object.
/// It converts each parameter into a String, using the debug_name if available,
/// otherwise using the ID field
#[macro_export]
macro_rules! extract_parameters {
    ($args:expr) => {
        $args
            .iter()
            .map(|var_id| {
                if let Some(debug_name) = &var_id.debug_name {
                    // If debug_name exists, use it as parameter
                    debug_name.clone().into()
                } else {
                    // If debug_name is None, use id field as parameter
                    format!("v{}", var_id.id)
                }
            })
            .collect::<Vec<String>>()
    };
}

/// Macro to convert a single var_id into its name using it id or debug name.
#[macro_export]
macro_rules! var_id_to_name {
    ($var_id:expr) => {
        if let Some(debug_name) = &$var_id.debug_name {
            // If debug_name exists, use it as parameter
            debug_name.clone().into()
        } else {
            // If debug_name is None, use id field as parameter
            format!("v{}", $var_id.id)
        }
    };
}
