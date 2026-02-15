use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use reqwest::Client as HttpClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::gemini;
use tracing::{info, warn};

/// GeminiClient interacts with Google Gemini API.
/// Text requests route through `rig-core`; image requests use direct REST.
/// Supports primary/fallback model pattern.
/// Mirrors Go's `GeminiClient` from `internal/ai/gemini_client.go`.
pub struct GeminiClient {
    client: gemini::Client,
    http_client: HttpClient,
    api_key: String,
    primary_model: String,
    fallback_model: String,
}

impl GeminiClient {
    /// Creates a new Gemini API client with primary/fallback model support.
    pub fn new(api_key: &str, primary_model: &str, fallback_model: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(anyhow!("GEMINI_API_KEY is empty"));
        }

        let client = gemini::Client::new(api_key)
            .map_err(|e| anyhow!("Failed to create Gemini client: {}", e))?;

        let primary = if primary_model.is_empty() {
            "gemini-2.5-flash"
        } else {
            primary_model
        };

        let fallback = if fallback_model.is_empty() {
            "gemini-2.0-flash"
        } else {
            fallback_model
        };

        info!(
            "AI: Gemini client initialized (primary={}, fallback={})",
            primary, fallback
        );

        Ok(Self {
            client,
            http_client: HttpClient::new(),
            api_key: api_key.to_string(),
            primary_model: primary.to_string(),
            fallback_model: fallback.to_string(),
        })
    }

    /// Sends a text prompt to Gemini with primary->fallback routing.
    /// Uses rig-core agent abstraction.
    /// Mirrors Go's `GenerateContent`.
    pub async fn generate_content(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        match self
            .generate_with_model(&self.primary_model, system_prompt, user_prompt)
            .await
        {
            Ok(response) => Ok(response),
            Err(primary_err) => {
                warn!(
                    "AI: Primary model ({}) failed: {}. Switching to fallback ({})...",
                    self.primary_model, primary_err, self.fallback_model
                );
                self.generate_with_model(&self.fallback_model, system_prompt, user_prompt)
                    .await
                    .map_err(|fallback_err| {
                        anyhow!(
                            "Both models failed. Primary: {}. Fallback: {}",
                            primary_err,
                            fallback_err
                        )
                    })
            }
        }
    }

    /// Helper: text generation via rig-core agent.
    async fn generate_with_model(
        &self,
        model_name: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let agent = self
            .client
            .agent(model_name)
            .preamble(system_prompt)
            .build();

        let response: String = agent
            .prompt(user_prompt)
            .await
            .map_err(|e| anyhow!("Gemini model '{}' error: {}", model_name, e))?;

        Ok(response)
    }

    /// Sends an image + prompt to Gemini for multimodal analysis.
    /// Uses direct REST call (Gemini inlineData format).
    /// Mirrors Go's `GenerateContentFromImage`.
    pub async fn generate_content_from_image(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        mime_type: &str,
        image_data: &[u8],
    ) -> Result<String> {
        match self
            .generate_image_with_model(
                &self.primary_model,
                system_prompt,
                user_prompt,
                mime_type,
                image_data,
            )
            .await
        {
            Ok(res) => Ok(res),
            Err(primary_err) => {
                warn!(
                    "AI: Primary vision model ({}) failed: {}. Switching to fallback ({})...",
                    self.primary_model, primary_err, self.fallback_model
                );
                self.generate_image_with_model(
                    &self.fallback_model,
                    system_prompt,
                    user_prompt,
                    mime_type,
                    image_data,
                )
                .await
                .map_err(|fallback_err| {
                    anyhow!(
                        "Both vision models failed. Primary: {}. Fallback: {}",
                        primary_err,
                        fallback_err
                    )
                })
            }
        }
    }

    /// Helper: multimodal generation via direct Gemini REST API.
    async fn generate_image_with_model(
        &self,
        model_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        mime_type: &str,
        image_data: &[u8],
    ) -> Result<String> {
        let b64_data = b64.encode(image_data);

        let payload = serde_json::json!({
            "systemInstruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": [{
                "parts": [
                    { "text": user_prompt },
                    {
                        "inlineData": {
                            "mimeType": mime_type,
                            "data": b64_data
                        }
                    }
                ]
            }]
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_name, self.api_key
        );

        let res = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Gemini vision API error ({}): {}",
                status,
                err_text
            ));
        }

        let body: serde_json::Value = res.json().await?;

        body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No text in Gemini vision response"))
    }
}
