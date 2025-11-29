use anyhow::{Context, Result, anyhow};
use std::fs;
use std::str::FromStr;
use toml::Value;

/// Help format options for displaying help text.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HelpFormat {
    #[default]
    Plain,
    Html,
    Markdown,
}

impl FromStr for HelpFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(HelpFormat::Plain),
            "html" => Ok(HelpFormat::Html),
            "markdown" | "md" => Ok(HelpFormat::Markdown),
            _ => Err(anyhow!(
                "Invalid help format '{}'. Valid options are: plain, html, markdown",
                s
            )),
        }
    }
}

impl std::fmt::Display for HelpFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpFormat::Plain => write!(f, "plain"),
            HelpFormat::Html => write!(f, "html"),
            HelpFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Configuration for bot message filtering.
#[derive(Debug, Clone)]
pub struct BotFilteringConfig {
    /// Whether to ignore messages from bot itself
    pub ignore_self: bool,
    /// Whether to ignore messages from users with "bot" in their username
    pub ignore_bots: bool,
    /// Specific list of user IDs to ignore
    pub ignored_users: Vec<String>,
}

/// Configuration for join detection.
#[derive(Debug, Clone)]
pub struct JoinDetectionConfig {
    /// Whether to detect user joins at all
    pub enabled: bool,
    /// Specific list of room IDs to monitor for joins (empty = all rooms)
    pub monitored_rooms: Vec<String>,
    /// Whether to send a welcome message to users who join
    pub send_welcome: bool,
    /// Welcome message to send to new users
    pub welcome_message: String,
    /// Format for the welcome message (plain, html, markdown)
    pub welcome_format: HelpFormat,
    /// Timeout in seconds for deduplication of welcome messages
    pub welcome_timeout_seconds: u64,
}

impl Default for BotFilteringConfig {
    fn default() -> Self {
        Self {
            ignore_self: true,
            ignore_bots: false,
            ignored_users: Vec::new(),
        }
    }
}

impl Default for JoinDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitored_rooms: Vec::new(),
            send_welcome: false,
            welcome_message: "Welcome to the room! Type !help for assistance.".to_string(),
            welcome_format: HelpFormat::Plain,
            welcome_timeout_seconds: 300,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub homeserver: String,
    pub username: String,
    pub access_token: String,
    pub log_file: String,
    pub working_dir: String,
    pub help_file: String,
    pub help_format: HelpFormat,
    pub bot_filtering: BotFilteringConfig,
    pub join_detection: JoinDetectionConfig,
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
            help_format: config
                .get("help_format")
                .and_then(|v| v.as_str())
                .map(HelpFormat::from_str)
                .transpose()?
                .unwrap_or_default(),
            bot_filtering: parse_bot_filtering_config(&config)?,
            join_detection: parse_join_detection_config(&config)?,
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
        println!("  Help Format: {}", self.help_format);
        println!("  Bot Filtering:");
        println!("    Ignore Self: {}", self.bot_filtering.ignore_self);
        println!("    Ignore Bots: {}", self.bot_filtering.ignore_bots);
        if !self.bot_filtering.ignored_users.is_empty() {
            println!("    Ignored Users:");
            for user in &self.bot_filtering.ignored_users {
                println!("      {}", user);
            }
        } else {
            println!("    Ignored Users: [none]");
        }
        println!("  Join Detection:");
        println!("    Enabled: {}", self.join_detection.enabled);
        if !self.join_detection.monitored_rooms.is_empty() {
            println!("    Monitored Rooms:");
            for room in &self.join_detection.monitored_rooms {
                println!("      {}", room);
            }
        } else {
            println!("    Monitored Rooms: [all rooms]");
        }
        println!("    Send Welcome: {}", self.join_detection.send_welcome);
        if self.join_detection.send_welcome {
            println!(
                "    Welcome Message: {}",
                self.join_detection.welcome_message
            );
            println!("    Welcome Format: {}", self.join_detection.welcome_format);
            println!(
                "    Welcome Timeout: {} seconds",
                self.join_detection.welcome_timeout_seconds
            );
        }
    }
}

/// Parse bot filtering configuration from TOML value.
fn parse_bot_filtering_config(config: &Value) -> Result<BotFilteringConfig> {
    let bot_filtering_config = config.get("bot_filtering");

    if let Some(bot_config) = bot_filtering_config {
        // Parse ignore_self
        let ignore_self = bot_config
            .get("ignore_self")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Parse ignore_bots
        let ignore_bots = bot_config
            .get("ignore_bots")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse ignored_users
        let ignored_users = bot_config
            .get("ignored_users")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(BotFilteringConfig {
            ignore_self,
            ignore_bots,
            ignored_users,
        })
    } else {
        // No bot_filtering section, use defaults
        Ok(BotFilteringConfig::default())
    }
}

/// Parse join detection configuration from TOML value.
fn parse_join_detection_config(config: &Value) -> Result<JoinDetectionConfig> {
    let join_detection_config = config.get("join_detection");

    if let Some(join_config) = join_detection_config {
        // Parse enabled
        let enabled = join_config
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Parse monitored_rooms
        let monitored_rooms = join_config
            .get("monitored_rooms")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Parse send_welcome
        let send_welcome = join_config
            .get("send_welcome")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse welcome_message
        let welcome_message = join_config
            .get("welcome_message")
            .and_then(|v| v.as_str())
            .unwrap_or("Welcome to the room! Type !help for assistance.")
            .to_string();

        // Parse welcome_format
        let welcome_format = join_config
            .get("welcome_format")
            .and_then(|v| v.as_str())
            .map(HelpFormat::from_str)
            .transpose()?
            .unwrap_or_default();

        // Parse welcome_timeout_seconds
        let welcome_timeout_seconds = join_config
            .get("welcome_timeout_seconds")
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
            .unwrap_or(300);

        Ok(JoinDetectionConfig {
            enabled,
            monitored_rooms,
            send_welcome,
            welcome_message,
            welcome_format,
            welcome_timeout_seconds,
        })
    } else {
        // No join_detection section, use defaults
        Ok(JoinDetectionConfig::default())
    }
}

/// Load help text from a file.
pub fn load_help_text(file_path: &str) -> Result<String> {
    fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read help file '{}'", file_path))
}

/// Check if a user ID should be ignored based on bot filtering configuration.
pub fn should_ignore_user(user_id: &str, bot_user_id: &str, config: &BotFilteringConfig) -> bool {
    // Check if it's bot itself
    if config.ignore_self && user_id == bot_user_id {
        return true;
    }

    // Check if user is in ignored list
    if config.ignored_users.contains(&user_id.to_string()) {
        return true;
    }

    // Check if user has "bot" in their username (case-insensitive)
    if config.ignore_bots && user_id.to_lowercase().contains("bot") {
        return true;
    }

    false
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
        assert_eq!(config.help_format, HelpFormat::Plain);
        // Bot filtering should use defaults when not specified
        assert!(config.bot_filtering.ignore_self);
        assert!(!config.bot_filtering.ignore_bots);
        assert!(config.bot_filtering.ignored_users.is_empty());
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
            help_format = \"markdown\"

            [bot_filtering]
            ignore_self = false
            ignore_bots = true
            ignored_users = [\"@spam-bot:example.com\", \"@announcement-bot:example.com\"]
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
        assert_eq!(config.help_format, HelpFormat::Markdown);
        assert!(!config.bot_filtering.ignore_self);
        assert!(config.bot_filtering.ignore_bots);
        assert_eq!(config.bot_filtering.ignored_users.len(), 2);
        assert!(
            config
                .bot_filtering
                .ignored_users
                .contains(&"@spam-bot:example.com".to_string())
        );
        assert!(
            config
                .bot_filtering
                .ignored_users
                .contains(&"@announcement-bot:example.com".to_string())
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing 'homeserver'")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing 'username'")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing 'access_token'")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing 'help_file'")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse TOML")
        );
    }

    #[test]
    fn test_help_format_parsing() {
        // Given TOML configurations with different help formats
        let plain_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
            help_format = \"plain\"
        "};

        let html_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
            help_format = \"html\"
        "};

        let markdown_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
            help_format = \"markdown\"
        "};

        let md_short_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
            help_format = \"md\"
        "};

        let invalid_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
            help_format = \"invalid\"
        "};

        // When parsing the configurations
        let plain_config = Config::from_toml(plain_toml).unwrap();
        let html_config = Config::from_toml(html_toml).unwrap();
        let markdown_config = Config::from_toml(markdown_toml).unwrap();
        let md_short_config = Config::from_toml(md_short_toml).unwrap();
        let invalid_result = Config::from_toml(invalid_toml);

        // Then valid formats should parse correctly
        assert_eq!(plain_config.help_format, HelpFormat::Plain);
        assert_eq!(html_config.help_format, HelpFormat::Html);
        assert_eq!(markdown_config.help_format, HelpFormat::Markdown);
        assert_eq!(md_short_config.help_format, HelpFormat::Markdown);

        // And invalid format should return error
        assert!(invalid_result.is_err());
        assert!(
            invalid_result
                .unwrap_err()
                .to_string()
                .contains("Invalid help format")
        );
    }

    #[test]
    fn test_load_help_text_success() {
        // Given a temporary file with help text content
        let help_content = "This is test help content";
        let temp_file = "test_help.txt";
        std::fs::write(temp_file, help_content).unwrap();

        // When loading help text from file
        let result = load_help_text(temp_file);

        // Then it should successfully load content
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), help_content);

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_help_text_file_not_found() {
        // Given a non-existent file path
        let non_existent_file = "non_existent_help.txt";

        // When trying to load help text from non-existent file
        let result = load_help_text(non_existent_file);

        // Then it should return an error
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read help file")
        );
    }

    #[test]
    fn test_should_ignore_user_self_filtering() {
        // Given bot filtering config with ignore_self = true
        let config = BotFilteringConfig {
            ignore_self: true,
            ignore_bots: false,
            ignored_users: vec![],
        };
        let bot_user_id = "@help-bot:example.com";
        let other_user_id = "@user:example.com";

        // When checking if bot should ignore its own messages
        assert!(should_ignore_user(bot_user_id, bot_user_id, &config));
        // When checking if bot should ignore other user's messages
        assert!(!should_ignore_user(other_user_id, bot_user_id, &config));
    }

    #[test]
    fn test_should_ignore_user_bot_pattern() {
        // Given bot filtering config with ignore_bots = true
        let config = BotFilteringConfig {
            ignore_self: false,
            ignore_bots: true,
            ignored_users: vec![],
        };
        let bot_user_id = "@help-bot:example.com";
        let other_bot_id = "@spam-bot:example.com";
        let regular_user_id = "@user:example.com";

        // When checking different user types
        assert!(should_ignore_user(bot_user_id, bot_user_id, &config)); // contains "bot" even though ignore_self is false
        assert!(should_ignore_user(other_bot_id, bot_user_id, &config)); // contains "bot"
        assert!(!should_ignore_user(regular_user_id, bot_user_id, &config)); // doesn't contain "bot"
    }

    #[test]
    fn test_should_ignore_user_specific_list() {
        // Given bot filtering config with specific ignored users
        let config = BotFilteringConfig {
            ignore_self: false,
            ignore_bots: false,
            ignored_users: vec![
                "@spam-bot:example.com".to_string(),
                "@announcement-bot:example.com".to_string(),
            ],
        };
        let bot_user_id = "@help-bot:example.com";
        let spam_bot_id = "@spam-bot:example.com";
        let announcement_bot_id = "@announcement-bot:example.com";
        let regular_user_id = "@user:example.com";

        // When checking different users
        assert!(!should_ignore_user(bot_user_id, bot_user_id, &config));
        assert!(should_ignore_user(spam_bot_id, bot_user_id, &config));
        assert!(should_ignore_user(
            announcement_bot_id,
            bot_user_id,
            &config
        ));
        assert!(!should_ignore_user(regular_user_id, bot_user_id, &config));
    }

    #[test]
    fn test_should_ignore_user_case_insensitive() {
        // Given bot filtering config with ignore_bots = true
        let config = BotFilteringConfig {
            ignore_self: false,
            ignore_bots: true,
            ignored_users: vec![],
        };
        let bot_user_id = "@help-bot:example.com";
        let uppercase_bot_id = "@HELP-BOT:example.com";
        let mixed_case_bot_id = "@Help-Bot:example.com";

        // When checking case-insensitive bot detection
        assert!(should_ignore_user(uppercase_bot_id, bot_user_id, &config));
        assert!(should_ignore_user(mixed_case_bot_id, bot_user_id, &config));
    }

    #[test]
    fn test_join_detection_config_parsing() {
        // Given TOML configurations with different join detection settings
        let enabled_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"

            [join_detection]
            enabled = true
            monitored_rooms = [\"!room1:example.com\", \"!room2:example.com\"]
            send_welcome = true
            welcome_message = \"Welcome! Type !help for assistance.\"
            welcome_format = \"markdown\"
        "};

        let disabled_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"

            [join_detection]
            enabled = false
        "};

        let all_rooms_toml = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"
        "};

        // When parsing the configurations
        let enabled_config = Config::from_toml(enabled_toml).unwrap();
        let disabled_config = Config::from_toml(disabled_toml).unwrap();
        let all_rooms_config = Config::from_toml(all_rooms_toml).unwrap();

        // Then configurations should be parsed correctly
        assert!(enabled_config.join_detection.enabled);
        assert_eq!(enabled_config.join_detection.monitored_rooms.len(), 2);
        assert!(
            enabled_config
                .join_detection
                .monitored_rooms
                .contains(&"!room1:example.com".to_string())
        );
        assert!(
            enabled_config
                .join_detection
                .monitored_rooms
                .contains(&"!room2:example.com".to_string())
        );
        assert!(enabled_config.join_detection.send_welcome);
        assert_eq!(
            enabled_config.join_detection.welcome_message,
            "Welcome! Type !help for assistance."
        );
        assert_eq!(
            enabled_config.join_detection.welcome_format,
            HelpFormat::Markdown
        );

        assert!(!disabled_config.join_detection.enabled);
        assert!(disabled_config.join_detection.monitored_rooms.is_empty());
        assert!(!disabled_config.join_detection.send_welcome);

        assert!(all_rooms_config.join_detection.enabled);
        assert!(all_rooms_config.join_detection.monitored_rooms.is_empty());
        assert!(!all_rooms_config.join_detection.send_welcome);
        assert_eq!(
            all_rooms_config.join_detection.welcome_message,
            "Welcome to the room! Type !help for assistance."
        );
        assert_eq!(
            all_rooms_config.join_detection.welcome_format,
            HelpFormat::Plain
        );
    }

    #[test]
    fn test_join_detection_config_with_timeout() {
        // Given TOML configuration with custom welcome timeout
        let toml_str = indoc! {"
            homeserver = \"https://matrix.example.com\"
            username = \"@bot:example.com\"
            access_token = \"secret_token\"
            help_file = \"help.md\"

            [join_detection]
            enabled = true
            send_welcome = true
            welcome_timeout_seconds = 600
        "};

        // When parsing the configuration
        let config = Config::from_toml(toml_str).unwrap();

        // Then the timeout should be parsed correctly
        assert_eq!(config.join_detection.welcome_timeout_seconds, 600);
    }
}
