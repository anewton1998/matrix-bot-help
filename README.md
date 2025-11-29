# Matrix Bot Help

A Matrix bot that provides help messages to users in Matrix rooms. The bot responds to `!help` commands and supports multiple help text formats (Plain, HTML, Markdown).

## Features

- **Multiple Help Formats**: Supports Plain text, HTML, and Markdown help messages
- **Bot Filtering**: Configurable filtering of bot messages and specific users
- **Auto-join**: Automatically joins rooms when invited
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

3. **Create help file:**
   ```bash
   cp bot-help.md.example bot-help.md
   # Customize the help content as needed
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

4. **Run with daemon mode:**
   ```bash
   docker run -d \
     --name matrix-bot-help \
     -v $(pwd)/config:/app/config \
     -v $(pwd)/data:/app/data \
     matrix-bot-help --daemonize
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
```

### Help Format Options

- **Plain**: Simple text formatting (example: `bot-help.txt.example`)
- **HTML**: Rich HTML formatting with CSS styling (example: `bot-help.html.example`)
- **Markdown**: Markdown formatting (example: `bot-help.md.example`)

## Command Line Options

```bash
matrix-bot-help [OPTIONS]

OPTIONS:
    -c, --config <FILE>    Config file path [default: bot.toml]
    -d, --daemonize        Daemonize the process [default: false]
    -h, --help             Print help information
```

## Docker Deployment

### Environment Variables

The Docker image supports the following environment variables:

- `CONFIG_FILE`: Path to configuration file (default: `/app/config/bot.toml`)
- `LOG_FILE`: Path to log file (default: `/app/data/bot.log`)

### Docker Compose Example

```yaml
version: '3.8'

services:
  matrix-bot-help:
    build: .
    container_name: matrix-bot-help
    restart: unless-stopped
    volumes:
      - ./config:/app/config:ro
      - ./data:/app/data
    environment:
      - CONFIG_FILE=/app/config/bot.toml
    command: ["--config", "/app/config/bot.toml", "--daemonize"]
```

### Production Docker Tips

1. **Use read-only config volume:**
   ```bash
   -v ./config:/app/config:ro
   ```

2. **Set proper file permissions:**
   ```bash
   chown 1001:1001 config/bot.toml
   chmod 600 config/bot.toml
   ```

3. **Use health checks:**
   ```bash
   docker ps --format "table {{.Names}}\t{{.Status}}"
   ```

4. **Log management:**
   ```bash
   docker logs -f matrix-bot-help
   docker logs --tail 100 matrix-bot-help
   ```

## Bot Filtering

The bot can be configured to ignore messages from:

- **Itself**: Set `ignore_self = true` to ignore bot's own messages
- **Other bots**: Set `ignore_bots = true` to ignore users with "bot" in their username
- **Specific users**: Add user IDs to `ignored_users` array

## Development

### Building from Source

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

### Project Structure

```
matrix-bot-help/
├── src/
│   ├── lib.rs          # Core library with Config and HelpFormat
│   └── main.rs         # Main application and bot logic
├── bot.toml.example    # Example configuration
├── bot-help.md.example # Example Markdown help file
├── bot-help.html.example # Example HTML help file
├── bot-help.txt.example  # Example plain text help file
├── Dockerfile          # Multi-stage Docker build
├── Cargo.toml          # Rust dependencies
└── README.md           # This file
```

## Security Considerations

1. **Access Tokens**: Store access tokens securely and never commit them to version control
2. **File Permissions**: Use restrictive file permissions for configuration files
3. **Container Security**: The Docker image runs as a non-root user (UID 1001)
4. **Network Security**: Consider using HTTPS for Matrix homeserver connections

## Troubleshooting

### Common Issues

1. **Authentication failures**: Verify access token and user ID are correct
2. **Permission errors**: Check file permissions for config and log files
3. **Network issues**: Ensure Matrix homeserver is accessible
4. **Container issues**: Check Docker logs with `docker logs matrix-bot-help`

### Debug Mode

For debugging, you can run without daemonization:

```bash
# Direct execution
./target/release/matrix-bot-help --config bot.toml

# Docker with logs
docker run --rm -v $(pwd)/config:/app/config matrix-bot-help
```

## License

This project is dual-licensed under the Apache License 2.0 and MIT License. See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files for details.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust conventions
- Tests pass (`cargo test`)
- Documentation is updated
- Commit messages are clear and descriptive

## Support

For issues and questions:
1. Check the troubleshooting section above
2. Review the example configuration files
3. Open an issue on the project repository
