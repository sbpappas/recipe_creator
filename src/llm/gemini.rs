use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::recipes::prompt::RecipeRequest;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Gemini quota exceeded. Your current free-tier allowance for this model has been exhausted. Try again later or switch to a different model.")]
    QuotaExceeded,
    #[error("Gemini returned an unexpected response. Try again.")]
    BadResponse,
    #[error("Could not parse recipe from Gemini. Try again.")]
    InvalidJson,
    #[error("Network error talking to Gemini: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Gemini error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct GeminiClient {
    http: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(http: Client, api_key: String, model: String) -> Self {
        Self {
            http,
            api_key,
            model,
        }
    }

    pub async fn generate_recipe(
        &self,
        request: &RecipeRequest,
    ) -> Result<GeneratedRecipe, LlmError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = GeminiRequest {
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: system_prompt(),
                }],
            }),
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: user_prompt(request),
                }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
            },
        };

        let response = self.http.post(&url).json(&body).send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        let text = String::from_utf8_lossy(&bytes).into_owned();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LlmError::QuotaExceeded);
        }

        if !status.is_success() {
            return Err(LlmError::Api(format!("Gemini error: {}", describe_response(status, &text))));
        }

        let payload: GeminiResponse = serde_json::from_slice(&bytes).map_err(|_| LlmError::BadResponse)?;

        let text = payload
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or(LlmError::BadResponse)?;

        serde_json::from_str(&text).map_err(|_| LlmError::InvalidJson)
    }
}

fn system_prompt() -> String {
    [
        "You are a helpful home cook assistant.",
        "Create practical recipes using the user's pantry staples when possible.",
        "Only suggest extra shopping items when truly necessary.",
        "Respect the meal type, must-use ingredients, and any notes.",
        "Respond with valid JSON only, matching the requested schema.",
    ]
    .join(" ")
}

fn user_prompt(request: &RecipeRequest) -> String {
    format!(
        "Create a recipe with this context:\n{}\n\nReturn JSON with this schema:\n{}",
        serde_json::to_string_pretty(request).unwrap_or_default(),
        RECIPE_SCHEMA
    )
}

fn describe_response(status: reqwest::StatusCode, body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {trimmed}")
    }
}

const RECIPE_SCHEMA: &str = r#"{
  "title": "string",
  "servings": number,
  "prep_minutes": number,
  "cook_minutes": number,
  "ingredients": [{"item": "string", "amount": "string"}],
  "steps": ["string"],
  "pantry_used": ["string"],
  "shopping_optional": ["string"]
}"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedRecipe {
    pub title: String,
    pub servings: u32,
    pub prep_minutes: u32,
    pub cook_minutes: u32,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<String>,
    pub pantry_used: Vec<String>,
    pub shopping_optional: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub item: String,
    pub amount: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[cfg(test)]
mod tests {
    use super::GeneratedRecipe;

    #[test]
    fn generated_recipe_deserializes_from_json() {
        let json = r#"{
            "title": "Garlic Rice Bowl",
            "servings": 2,
            "prep_minutes": 5,
            "cook_minutes": 15,
            "ingredients": [{"item": "rice", "amount": "1 cup"}],
            "steps": ["Cook rice", "Serve"],
            "pantry_used": ["rice", "salt"],
            "shopping_optional": []
        }"#;

        let recipe: GeneratedRecipe = serde_json::from_str(json).expect("valid recipe json");
        assert_eq!(recipe.title, "Garlic Rice Bowl");
        assert_eq!(recipe.ingredients.len(), 1);
    }

    #[test]
    fn describe_response_includes_status_and_body() {
        let message = super::describe_response(reqwest::StatusCode::TOO_MANY_REQUESTS, r#"{"error":{"message":"Quota exceeded"}}"#);

        assert!(message.contains("429"));
        assert!(message.contains("Quota exceeded"));
    }
}
