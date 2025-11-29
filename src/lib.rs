use anyhow::{anyhow, Context, Result};
use std::fs;
use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub homeserver: String,
    pub username: String,
    pub access_token: String,
    pub log_file: String,
    pub working_dir: String,
    pub help_file: String,
}

/// Load help text from a file.
pub fn load_help_text(file_path: &str) -> Result<String> {
    fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read help file '{}'", file_path))
}

impl Config {
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let config: Value =
            toml::from_str(toml_str).map_err(|e| anyhow!("Failed to parse TOML: {}", e))?;

        Ok(Config {
            homeserver: config
                .get("homeserver")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'homeserver' in config file"))?
                .to_string(),
            username: config
                .get("username")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'username' in config file"))?
                .to_string(),
            access_token: config
                .get("access_token")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'access_token' in config file"))?
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
            help_file: config
                .get("help_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'help_file' in config file"))?
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
        println!("  Help File: {}", self.help_file);
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
            help_file = \"help.md\"
        "};

        // When parsing the TOML configuration
        let config = Config::from_toml(toml_str).unwrap();

        // Then all required fields should be parsed correctly and defaults should be applied
        assert_eq!(config.homeserver, "https://matrix.example.com");
        assert_eq!(config.username, "@bot:example.com");
        assert_eq!(config.access_token, "secret_token");
        assert_eq!(config.log_file, "bot.log");
        assert_eq!(config.working_dir, ".");
        assert_eq!(config.help_file, "help.md");
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
            help_file = \"/path/to/help.md\"
        "};

        // When parsing the TOML configuration
        let config = Config::from_toml(toml_str).unwrap();

        // Then all fields should be parsed with their specified values
        assert_eq!(config.homeserver, "https://matrix.example.com");
        assert_eq!(config.username, "@bot:example.com");
        assert_eq!(config.access_token, "secret_token");
        assert_eq!(config.log_file, "/var/log/bot.log");
        assert_eq!(config.working_dir, "/app");
        assert_eq!(config.help_file, "/path/to/help.md");
    }

    #[test]
    fn test_missing_homeserver_error() {
        // Given a TOML configuration missing the homeserver field
        let toml_str = indoc! {"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'homeserver'"));
    }

    #[test]
    fn test_missing_username_error() {
        // Given a TOML configuration missing the username field
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'username'"));
    }

    #[test]
    fn test_missing_access_token_error() {
        // Given a TOML configuration missing the access_token field
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            help_file = \"help.md\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'access_token'"));
    }

    #[test]
    fn test_missing_help_file_error() {
        // Given a TOML configuration missing the help_file field
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
        "};

        // When parsing the TOML configuration
        let result = Config::from_toml(toml_str);

        // Then it should return an error indicating the missing field
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'help_file'"));
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
        assert!(result.unwrap_err().to_string().contains("Failed to parse TOML"));
    }

    #[test]
    fn test_load_help_text_success() {
        // Given a temporary file with help text content
        let help_content = "This is test help content";
        let temp_file = "test_help.txt";
        std::fs::write(temp_file, help_content).unwrap();

        // When loading help text from the file
        let result = load_help_text(temp_file);

        // Then it should successfully load the content
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), help_content);

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_help_text_file_not_found() {
        // Given a non-existent file path
        let non_existent_file = "non_existent_help.txt";

        // When trying to load help text from the non-existent file
        let result = load_help_text(non_existent_file);

        // Then it should return an error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read help file"));
    }
}
