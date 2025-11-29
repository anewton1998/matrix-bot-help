use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub homeserver: String,
    pub username: String,
    pub access_token: String,
    pub log_file: String,
    pub working_dir: String,
    pub help_text: String,
}

impl Config {
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        let config: Value =
            toml::from_str(toml_str).map_err(|e| format!("Failed to parse TOML: {}", e))?;

        Ok(Config {
            homeserver: config
                .get("homeserver")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'homeserver' in config file")?
                .to_string(),
            username: config
                .get("username")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'username' in config file")?
                .to_string(),
            access_token: config
                .get("access_token")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'access_token' in config file")?
                .to_string(),
            log_file: config
                .get("log_file")
                .and_then(|v| v.as_str())
                .unwrap_or("bot.log")
                .to_string(),
            working_dir: config
                .get("working_directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".")
                .to_string(),
            help_text: config
                .get("help_text")
                .and_then(|v| v.as_str())
                .unwrap_or("No help text configured")
                .to_string(),
        })
    }

    pub fn print(&self) {
        println!("Configuration:");
        println!("  Homeserver: {}", self.homeserver);
        println!("  Username: {}", self.username);
        println!(
            "  Access Token: {}",
            if self.access_token.is_empty() {
                "[empty]"
            } else {
                "[set]"
            }
        );
        println!("  Log File: {}", self.log_file);
        println!("  Working Directory: {}", self.working_dir);
        println!("  Help Text: {}", self.help_text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_minimal_config_parsing() {
        // Given a minimal TOML configuration with only required fields
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
        "};

        // When parsing the TOML configuration
        let config = Config::from_toml(toml_str).unwrap();

        // Then all required fields should be parsed correctly and defaults should be applied
        assert_eq!(config.homeserver, "https://matrix.example.com");
        assert_eq!(config.username, "@bot:example.com");
        assert_eq!(config.access_token, "secret_token");
        assert_eq!(config.log_file, "bot.log");
        assert_eq!(config.working_dir, ".");
        assert_eq!(config.help_text, "No help text configured");
    }

    #[test]
    fn test_full_config_parsing() {
        // Given a complete TOML configuration with all optional fields
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            log_file = \"/var/log/bot.log\"
            working_directory = \"/app\"
            help_text = \"\"\"
            This is a multiline help text.
            It can span multiple lines.
            \"\"\"
        "};

        // When parsing the TOML configuration
        let config = Config::from_toml(toml_str).unwrap();

        // Then all fields should be parsed with their specified values
        assert_eq!(config.homeserver, "https://matrix.example.com");
        assert_eq!(config.username, "@bot:example.com");
        assert_eq!(config.access_token, "secret_token");
        assert_eq!(config.log_file, "/var/log/bot.log");
        assert_eq!(config.working_dir, "/app");
        assert_eq!(
            config.help_text,
            "This is a multiline help text.\nIt can span multiple lines.\n"
        );
    }

    #[test]
    fn test_missing_homeserver_error() {
        // Given a TOML configuration missing the homeserver field
        let toml_str = indoc! {"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'homeserver'"));
    }

    #[test]
    fn test_missing_username_error() {
        // Given a TOML configuration missing the username field
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            access_token = \"secret_token\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'username'"));
    }

    #[test]
    fn test_missing_access_token_error() {
        // Given a TOML configuration missing the access_token field
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'access_token'"));
    }

    #[test]
    fn test_invalid_toml_syntax_error() {
        // Given a TOML configuration with invalid syntax
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            invalid_syntax = [
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating TOML parsing failure
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse TOML"));
    }
}
