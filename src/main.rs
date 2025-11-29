use clap::Parser;
use daemonize::Daemonize;
use matrix_bot_help::Config;
use std::fs::{self, OpenOptions};

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

    // Parse configuration from TOML
    let config = Config::from_toml(&config_content)
        .unwrap_or_else(|e| panic!("Failed to parse config: {}", e));

    println!("Config loaded:");
    config.print();

    // Daemonize if requested
    if cli.daemonize {
        let log_file_handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)
            .unwrap_or_else(|e| panic!("Failed to open log file '{}': {}", config.log_file, e));

        let daemonize = Daemonize::new()
            .pid_file("/tmp/matrix-bot-help.pid")
            .working_directory(&config.working_dir)
            .stdout(
                log_file_handle
                    .try_clone()
                    .unwrap_or_else(|e| panic!("Failed to clone log file handle: {}", e)),
            )
            .stderr(log_file_handle);

        match daemonize.start() {
            Ok(_) => {
                println!("Successfully daemonized, PID: {}", std::process::id());
                config.print();
            }
            Err(e) => eprintln!("Failed to daemonize: {}", e),
        }
    }
}
