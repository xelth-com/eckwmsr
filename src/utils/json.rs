/// Cleans raw AI output to extract valid JSON.
/// Removes Markdown code blocks (```json ... ```) and surrounding whitespace.
/// Mirrors Go's `SanitizeJSON` from `internal/utils/json.go`.
pub fn sanitize_json(input: &str) -> String {
    let mut cleaned = input.trim();

    if cleaned.starts_with("```json") {
        cleaned = &cleaned[7..];
    } else if cleaned.starts_with("```") {
        cleaned = &cleaned[3..];
    }

    if cleaned.ends_with("```") {
        cleaned = &cleaned[..cleaned.len() - 3];
    }

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_block() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(sanitize_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_clean_generic_block() {
        let input = "```\n{\"key\": \"value\"}\n```";
        assert_eq!(sanitize_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_plain_json() {
        let input = r#"{"key": "value"}"#;
        assert_eq!(sanitize_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_whitespace() {
        let input = "  \n  {\"ok\": true}  \n  ";
        assert_eq!(sanitize_json(input), r#"{"ok": true}"#);
    }
}
