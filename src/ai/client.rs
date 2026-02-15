use anyhow::{anyhow, Result};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::gemini;
use tracing::{info, warn};

/// GeminiClient interacts with Google Gemini API using rig-core.
/// Supports primary/fallback model pattern.
/// Mirrors Go's `GeminiClient` from `internal/ai/gemini_client.go`.
pub struct GeminiClient {
    client: gemini::Client,
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
            primary_model: primary.to_string(),
            fallback_model: fallback.to_string(),
        })
    }

    /// Sends a prompt to Gemini with primary->fallback routing.
    /// Mirrors Go's `GenerateContent`.
    pub async fn generate_content(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        // Try primary model
        match self.generate_with_model(&self.primary_model, system_prompt, user_prompt).await {
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
                            "Both primary and fallback models failed. Primary: {}. Fallback: {}",
                            primary_err,
                            fallback_err
                        )
                    })
            }
        }
    }

    /// Helper to generate content with a specific model.
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
}
