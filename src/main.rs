use clap::Parser;
use std::fs;
use toml::Value;

#[derive(Parser)]
#[command(name = "matrix-bot-help")]
#[command(about = "A Matrix bot for help")]
struct Cli {
    /// Config file path
    #[arg(short, long, default_value = "bot.toml")]
    config: String,
    
    /// Daemonize the process
    #[arg(short = 'd', long, default_value = "false")]
    daemonize: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("Using config file: {}", cli.config);
    println!("Daemonize: {}", cli.daemonize);
    
    // Read and parse config file
    let config_content = fs::read_to_string(&cli.config)
        .unwrap_or_else(|e| panic!("Failed to read config file '{}': {}", cli.config, e));
    
    let config: Value = toml::from_str(&config_content)
        .unwrap_or_else(|e| panic!("Failed to parse config file '{}': {}", cli.config, e));
    
    // Extract required values from config
    let homeserver = config.get("homeserver")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Missing 'homeserver' in config file"));
    
    let username = config.get("username")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Missing 'username' in config file"));
    
    let access_token = config.get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Missing 'access_token' in config file"));
    
    println!("Config loaded:");
    println!("  Homeserver: {}", homeserver);
    println!("  Username: {}", username);
    println!("  Access Token: {}", if access_token.is_empty() { "[empty]" } else { "[set]" });
}
