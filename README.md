# Matrix Bot Help

A Matrix bot that provides help messages to users in Matrix rooms. The bot responds to `!help` commands and supports multiple help text formats (Plain, HTML, Markdown).

## Features

- **Multiple Help Formats**: Supports Plain text, HTML, and Markdown help messages
- **Bot Filtering**: Configurable filtering of bot messages and specific users
- **Auto-join**: Automatically joins rooms when invited
- **Welcome Messages**: Sends welcome messages when users join specific rooms, with support for custom welcome files
- **Daemon Mode**: Can run as a background daemon
- **Docker Support**: Containerized deployment with multi-stage builds
- **Configuration**: TOML-based configuration with sensible defaults

## Quick Start

### Using Cargo

1. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd matrix-bot-help
   cargo build --release
   ```

2. **Configure the bot:**
   ```bash
   cp bot.toml.example bot.toml
   # Edit bot.toml with your Matrix server details
   ```

3. **Create help and welcome files:**
    ```bash
    cp bot-help.md.example bot-help.md
    cp bot-welcome.md.example bot-welcome.md
    # Customize the help and welcome content as needed
    ```

4. **Run the bot:**
   ```bash
   ./target/release/matrix-bot-help --config bot.toml
   ```

### Using Docker

1. **Build the Docker image:**
   ```bash
   docker build -t matrix-bot-help .
   ```

2. **Prepare configuration:**
    ```bash
    mkdir -p config data
    cp bot.toml.example config/bot.toml
    cp bot-help.md.example config/bot-help.md
    cp bot-welcome.md.example config/bot-welcome.md
    # Edit config/bot.toml with your settings
    ```

3. **Run the container:**
   ```bash
   docker run -d \
     --name matrix-bot-help \
     -v $(pwd)/config:/app/config \
     -v $(pwd)/data:/app/data \
     matrix-bot-help
   ```

## Configuration

### Basic Configuration (bot.toml)

```toml
# Required fields
homeserver = "https://matrix.example.com"
username = "@help-bot:example.com"
access_token = "your_access_token_here"
help_file = "/app/config/bot-help.md"

# Optional fields
log_file = "/app/data/bot.log" # only used when deamonized
working_directory = "/app/data"
help_format = "markdown"  # Options: plain, html, markdown

# Bot filtering (optional)
[bot_filtering]
ignore_self = true
ignore_bots = false
ignored_users = ["@spam-bot:example.com"]

# Join detection and welcome messages (optional)
[join_detection]
enabled = true
monitored_rooms = ["!roomid:example.com", "!anotherroom:example.com"]
send_welcome = true
welcome_message = "Welcome to the room! Type !help for assistance."
welcome_file = "/app/config/bot-welcome.md"  # Optional: overrides/extends welcome_message
welcome_format = "markdown"  # Options: plain, html, markdown
welcome_timeout_seconds = 300
```

## Development

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo check
cargo clippy
```

## Troubleshooting

### Common Issues

1. **Authentication failures**: Verify access token and user ID are correct
2. **Permission errors**: Check file permissions for config and log files
3. **Network issues**: Ensure Matrix homeserver is correct (sometimes https://synapse.example.com instead of https://example.com)
4. **Container issues**: Check Docker logs with `docker logs matrix-bot-help`
5. **Missing files**: The bot will fail to start if `help_file` or `welcome_file` (if specified) don't exist

## License

This project is dual-licensed under the Apache License 2.0 and MIT License. See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files for details.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust conventions
- Tests pass (`cargo test`)
- Documentation is updated
- Commit messages are clear and descriptive

