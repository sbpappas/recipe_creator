use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeRequest {
    pub meal_type: String,
    pub pantry_staples: Vec<String>,
    pub must_use: Vec<String>,
    pub notes: Option<String>,
}

pub fn build_request(
    meal_type: &str,
    pantry_staples: Vec<String>,
    must_use_raw: &str,
    notes: Option<String>,
) -> RecipeRequest {
    let must_use = must_use_raw
        .split([',', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    RecipeRequest {
        meal_type: meal_type.to_string(),
        pantry_staples,
        must_use,
        notes: notes.filter(|n| !n.trim().is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_splits_must_use_ingredients() {
        let request = build_request(
            "dinner",
            vec!["salt".into(), "rice".into()],
            "chicken, spinach",
            Some("quick meal".into()),
        );

        assert_eq!(request.meal_type, "dinner");
        assert_eq!(request.must_use, vec!["chicken", "spinach"]);
    }
}
