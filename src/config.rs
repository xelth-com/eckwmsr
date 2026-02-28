use dotenvy::dotenv;
use sha2::{Digest, Sha256};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub instance_id: String,
    /// Human-readable name for this server (derived from BASE_URL or INSTANCE_NAME env)
    pub instance_name: String,
    pub base_url: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub sync_relay_url: String,
    pub sync_network_key: String,
    /// Derived from SYNC_NETWORK_KEY: sha256(key)[:16] (16 hex chars)
    pub mesh_id: String,
    pub gemini_api_key: String,
    pub gemini_primary_model: String,
    pub gemini_fallback_model: String,
    pub odoo: OdooConfig,
}

#[derive(Clone, Default)]
pub struct OdooConfig {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

pub fn load_config() -> Config {
    let _ = dotenv();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3210".to_string())
        .parse()
        .unwrap_or(3210);

    let instance_id = ensure_uuid_instance_id(
        &env::var("INSTANCE_ID").unwrap_or_else(|_| "rust_dev_node".to_string()),
    );

    let base_url = env::var("BASE_URL").unwrap_or_default();

    let instance_name = env::var("INSTANCE_NAME").unwrap_or_else(|_| {
        // Derive from BASE_URL hostname, or fall back to truncated UUID
        derive_hostname(&base_url).unwrap_or_else(|| instance_id.chars().take(8).collect())
    });

    let database_url = env::var("DATABASE_URL").unwrap_or_default();

    let jwt_secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_me".to_string());

    let sync_relay_url =
        env::var("SYNC_RELAY_URL").unwrap_or_else(|_| "https://9eck.com".to_string());

    // 32-byte hex key (64 hex chars). Zeroed placeholder for dev.
    let sync_network_key = env::var("SYNC_NETWORK_KEY").unwrap_or_else(|_| {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    });

    let mesh_id = compute_mesh_id(&sync_network_key);

    let gemini_api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    let gemini_primary_model =
        env::var("GEMINI_PRIMARY_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let gemini_fallback_model =
        env::var("GEMINI_FALLBACK_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());

    let odoo = OdooConfig {
        url: env::var("ODOO_URL").unwrap_or_default(),
        database: env::var("ODOO_DB").unwrap_or_default(),
        username: env::var("ODOO_USER").unwrap_or_default(),
        password: env::var("ODOO_PASSWORD").unwrap_or_default(),
    };

    Config {
        port,
        instance_id,
        instance_name,
        base_url,
        database_url,
        jwt_secret,
        sync_relay_url,
        sync_network_key,
        mesh_id,
        gemini_api_key,
        gemini_primary_model,
        gemini_fallback_model,
        odoo,
    }
}

/// Ensure instance_id is a valid UUID. If not, generate one and persist it to .env.
fn ensure_uuid_instance_id(raw: &str) -> String {
    if uuid::Uuid::parse_str(raw).is_ok() {
        return raw.to_string();
    }
    let id = uuid::Uuid::new_v4();
    // Persist to .env so the same UUID is used on next startup
    if let Ok(contents) = std::fs::read_to_string(".env") {
        let old_line = format!("INSTANCE_ID={}", raw);
        let new_line = format!("INSTANCE_ID={}", id);
        let updated = if contents.contains(&old_line) {
            contents.replace(&old_line, &new_line)
        } else if contents.contains("INSTANCE_ID=") {
            // Replace whatever value is there
            let mut result = String::new();
            for line in contents.lines() {
                if line.starts_with("INSTANCE_ID=") {
                    result.push_str(&new_line);
                } else {
                    result.push_str(line);
                }
                result.push('\n');
            }
            result
        } else {
            format!("{}\n{}\n", contents.trim_end(), new_line)
        };
        let _ = std::fs::write(".env", updated);
    }
    eprintln!("Generated INSTANCE_ID={} (saved to .env)", id);
    id.to_string()
}

/// Extract hostname from a URL string (e.g. "https://pda.repair/E" -> "pda.repair")
fn derive_hostname(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    // Strip scheme
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Take everything before first / or :
    let host = without_scheme.split(&['/', ':'][..]).next()?;
    if host.is_empty() || host == "localhost" {
        return None;
    }
    Some(host.to_string())
}

/// Compute mesh_id from SYNC_NETWORK_KEY.
/// mesh_id = sha256(key)[:8 bytes] = 16 hex characters
fn compute_mesh_id(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    hex::encode(&hash[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_id_deterministic() {
        let id1 = compute_mesh_id("test_key");
        let id2 = compute_mesh_id("test_key");
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn test_mesh_id_different_keys() {
        let id1 = compute_mesh_id("key_a");
        let id2 = compute_mesh_id("key_b");
        assert_ne!(id1, id2);
    }
}
