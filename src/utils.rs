use anyhow::Result;

/// Converts a ';'-seperated string to a vector of strings.
pub fn string_to_vec(s: &String) -> Vec<String> {
    s.split("; ")
        .filter_map(|s| match s.trim() {
            "" => None,
            s => Some(s.to_string()),
        })
        .collect()
}

/// Converts a vector of strings to a ';'-seperated string.
pub fn vec_to_string(v: &Vec<String>) -> String {
    v.join("; ")
}

/// Converts a string to an `Amount` and unit tuple.
/// Example: "1.0; kg" -> (1.0, "kg".to_string())
pub fn string_to_amount_unit(s: &Option<String>) -> Result<Option<Amount>> {
    match s {
        None => Ok(None),
        Some(s) => {
            let (quantity, unit) = s.split_once("; ").ok_or_else(|| {
                anyhow::anyhow!("Could not split string into quantity and unit: {}", s)
            })?;

            let quantity = quantity.parse()?;
            let unit = unit.to_string();
            Ok(Some((quantity, unit)))
        }
    }
}

/// Converts an `Amount` and unit tuple to a string.
/// Example: (1.0, "kg".to_string()) -> "1.0; kg"
pub fn amount_unit_to_string(s: &Option<Amount>) -> Option<String> {
    match s {
        Some((quantity, unit)) => Some(format!("{}; {}", quantity, unit)),
        None => None,
    }
}

/// Holds the quantity and unit of an item.
pub type Amount = (f32, String);
