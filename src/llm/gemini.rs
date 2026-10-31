use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::recipes::prompt::RecipeRequest;

/// How many times to try a request that fails with a temporary problem (Gemini busy, network blip).
const MAX_ATTEMPTS: u32 = 3;
/// Wait before each retry: 1s, then 3s.
const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(3)];
/// Recipes can take a while to generate, but don't hang forever.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

/// User-facing messages are kept short and free of raw API output; details go to the server log.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Gemini quota exceeded. Your current free-tier allowance for this model has been exhausted. Try again later or switch to a different model.")]
    QuotaExceeded,
    #[error("Gemini is busy right now. Please try again in a moment.")]
    Busy,
    #[error("Couldn't reach Gemini. Check your connection and try again.")]
    Network,
    #[error("Gemini returned an unexpected response. Try again.")]
    BadResponse,
    #[error("Could not parse recipe from Gemini. Try again.")]
    InvalidJson,
    /// A non-temporary error, e.g. an invalid API key or model name. Holds Google's one-line message.
    #[error("Gemini couldn't create a recipe (HTTP {status}): {message}")]
    Api { status: u16, message: String },
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
        // The key goes in a header, not the URL, so it can't leak through error messages or logs.
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
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

        let mut attempt = 1;
        let bytes = loop {
            let error = match self.send(&url, &body).await {
                Ok(bytes) => break bytes,
                Err(error) => error,
            };
            let retryable = matches!(error, LlmError::Busy | LlmError::Network);
            if !retryable || attempt >= MAX_ATTEMPTS {
                return Err(error);
            }
            let delay = RETRY_DELAYS[(attempt - 1) as usize];
            tracing::info!("gemini attempt {attempt} failed ({error}); retrying in {delay:?}");
            tokio::time::sleep(delay).await;
            attempt += 1;
        };

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

    /// Sends one request and returns the raw response body, or a classified error.
    async fn send(&self, url: &str, body: &GeminiRequest) -> Result<Vec<u8>, LlmError> {
        let network_error = |e: reqwest::Error| {
            tracing::warn!("gemini network error: {}", e.without_url());
            LlmError::Network
        };

        let response = self
            .http
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .timeout(REQUEST_TIMEOUT)
            .json(body)
            .send()
            .await
            .map_err(network_error)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(network_error)?;

        if status.is_success() {
            return Ok(bytes.to_vec());
        }

        let body_text = String::from_utf8_lossy(&bytes);
        tracing::warn!("gemini returned HTTP {status}: {}", body_text.trim());
        Err(classify_error(status, &body_text))
    }
}

/// Maps a failed HTTP response to an error. 5xx errors are temporary on Google's side.
fn classify_error(status: StatusCode, body: &str) -> LlmError {
    if status == StatusCode::TOO_MANY_REQUESTS {
        LlmError::QuotaExceeded
    } else if status.is_server_error() {
        LlmError::Busy
    } else {
        LlmError::Api {
            status: status.as_u16(),
            message: api_error_message(body).unwrap_or_else(|| {
                status.canonical_reason().unwrap_or("unknown error").to_string()
            }),
        }
    }
}

/// Pulls the human-readable message out of Google's error JSON: {"error": {"message": "..."}}.
fn api_error_message(body: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = json.get("error")?.get("message")?.as_str()?.trim();
    (!message.is_empty()).then(|| message.to_string())
}

fn system_prompt() -> String {
    [
        "You are a helpful home cook assistant.",
        "Create practical recipes using the user's pantry staples when possible.",
        "Only suggest extra shopping items when truly necessary.",
        "Respect the meal type, must-use ingredients, and any notes.",
        "Write every text value (title, ingredients, amounts, steps, lists) in the language given by the `language` field; keep the JSON keys in English.",
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
    fn classifies_errors_into_friendly_messages() {
        use super::{classify_error, LlmError};
        use reqwest::StatusCode;

        let overloaded = r#"{"error":{"code":503,"message":"This model is currently experiencing high demand.","status":"UNAVAILABLE"}}"#;
        assert!(matches!(classify_error(StatusCode::SERVICE_UNAVAILABLE, overloaded), LlmError::Busy));
        assert!(matches!(classify_error(StatusCode::INTERNAL_SERVER_ERROR, ""), LlmError::Busy));
        assert!(matches!(classify_error(StatusCode::TOO_MANY_REQUESTS, ""), LlmError::QuotaExceeded));

        let bad_key = r#"{"error":{"code":400,"message":"API key not valid. Please pass a valid API key.","status":"INVALID_ARGUMENT"}}"#;
        let error = classify_error(StatusCode::BAD_REQUEST, bad_key);
        assert_eq!(
            error.to_string(),
            "Gemini couldn't create a recipe (HTTP 400): API key not valid. Please pass a valid API key."
        );

        // No JSON body: fall back to the status text instead of showing nothing (or raw HTML).
        let error = classify_error(StatusCode::NOT_FOUND, "<html>not found</html>");
        assert_eq!(error.to_string(), "Gemini couldn't create a recipe (HTTP 404): Not Found");
    }
}
