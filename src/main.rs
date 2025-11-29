use anyhow::{Context, Result};
use clap::Parser;
use daemonize::Daemonize;
use matrix_bot_help::{Config, HelpFormat, load_help_text, should_ignore_user};
use matrix_sdk::{
    Client, Room, RoomState, SessionMeta, SessionTokens,
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::events::room::member::{MembershipState, StrippedRoomMemberEvent, SyncRoomMemberEvent},
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
    ruma::{UserId, device_id},
};
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("Using config file: {}", cli.config);
    println!("Daemonize: {}", cli.daemonize);

    // Read and parse config file
    let config_content = fs::read_to_string(&cli.config)
        .with_context(|| format!("Failed to read config file '{}'", cli.config))?;

    // Parse configuration from TOML
    let config = Config::from_toml(&config_content).context("Failed to parse config")?;

    println!("Config loaded:");
    config.print();

    // Daemonize if requested
    if cli.daemonize {
        let log_file_handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)
            .with_context(|| format!("Failed to open log file '{}'", config.log_file))?;

        let daemonize = Daemonize::new()
            .pid_file("/tmp/matrix-bot-help.pid")
            .working_directory(&config.working_dir)
            .stdout(
                log_file_handle
                    .try_clone()
                    .context("Failed to clone log file handle")?,
            )
            .stderr(log_file_handle);

        daemonize.start().context("Failed to daemonize")?;

        println!("Successfully daemonized, PID: {}", std::process::id());
        config.print();

        // Bot logic runs here after daemonizing
        run_bot(&config).await?;
    } else {
        // Non-daemon bot logic
        run_bot(&config).await?;
    }

    Ok(())
}

async fn run_bot(config: &Config) -> Result<()> {
    println!("Starting Matrix bot with homeserver: {}", config.homeserver);

    // Create client
    let client = Client::builder()
        .homeserver_url(&config.homeserver)
        .build()
        .await?;

    // Create a MatrixSession with existing access token
    let user_id = UserId::parse(&config.username)
        .map_err(|e| anyhow::anyhow!("Invalid user ID '{}': {}", config.username, e))?;

    let session = MatrixSession {
        meta: SessionMeta {
            user_id,
            device_id: device_id!("matrix-bot-help").to_owned(),
        },
        tokens: SessionTokens {
            access_token: config.access_token.clone(),
            refresh_token: None,
        },
    };

    // Restore the session with access token
    client
        .matrix_auth()
        .restore_session(session, matrix_sdk::store::RoomLoadSettings::default())
        .await?;

    println!("Successfully logged in as {}", config.username);

    // Initial sync to avoid responding to old messages
    let response = client.sync_once(SyncSettings::default()).await?;
    println!("Initial sync completed");

    // Load help text at startup
    let help_text = load_help_text(&config.help_file).context("Failed to load help text")?;

    // Get bot user ID for filtering
    let bot_user_id = client
        .user_id()
        .expect("Client should have a user ID")
        .to_owned();

    // Add event handler for room messages
    let bot_filtering = config.bot_filtering.clone();
    let help_format = config.help_format.clone();
    client.add_event_handler(
        move |event: OriginalSyncRoomMessageEvent, room: Room| async move {
            on_room_message(
                event,
                room,
                &help_text,
                &bot_user_id,
                &bot_filtering,
                &help_format,
            )
            .await
        },
    );

    // Add event handler for autojoining rooms when invited
    client.add_event_handler(on_stripped_state_member);

    // Add event handler for detecting when users join rooms
    let join_detection_config = config.join_detection.clone();
    client.add_event_handler(move |event: SyncRoomMemberEvent, room: Room| async move {
        on_room_member(event, room, &join_detection_config).await
    });

    // Start continuous sync
    let settings = SyncSettings::default().token(response.next_batch);
    println!("Starting continuous sync...");
    client.sync(settings).await?;

    Ok(())
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    help_text: &str,
    bot_user_id: &UserId,
    bot_filtering: &matrix_bot_help::BotFilteringConfig,
    help_format: &HelpFormat,
) {
    // Only respond to messages in joined rooms
    if room.state() != RoomState::Joined {
        return;
    }

    let MessageType::Text(text_content) = event.content.msgtype else {
        return;
    };

    // Check if sender should be ignored based on bot filtering configuration
    if should_ignore_user(event.sender.as_str(), bot_user_id.as_str(), bot_filtering) {
        println!("Ignoring message from filtered user: {}", event.sender);
        return;
    }

    // Check if message starts with help command
    if text_content.body.starts_with("!help") {
        println!("Received help request in room {}", room.room_id());

        let response = match help_format {
            HelpFormat::Plain => RoomMessageEventContent::text_plain(help_text),
            HelpFormat::Html => RoomMessageEventContent::text_html(help_text, help_text),
            HelpFormat::Markdown => RoomMessageEventContent::text_markdown(help_text),
        };

        if let Err(e) = room.send(response).await {
            eprintln!("Failed to send help message: {}", e);
        }
    }
}

async fn on_stripped_state_member(event: StrippedRoomMemberEvent, client: Client, room: Room) {
    // Only process invitations for the bot itself
    if event.state_key != client.user_id().expect("Client should have a user ID") {
        return;
    }

    // Check if this is an invitation
    if event.content.membership == MembershipState::Invite {
        println!("Received invitation to room {}", room.room_id());

        // Join the room with retry logic
        let room_id = room.room_id().to_owned();
        tokio::spawn(async move {
            let mut delay = 2;

            while let Err(e) = room.join().await {
                eprintln!(
                    "Failed to join room {} ({}), retrying in {}s",
                    room_id, e, delay
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    eprintln!("Can't join room {} after multiple retries", room_id);
                    break;
                }
            }

            if (room.join().await).is_ok() {
                println!("Successfully joined room {}", room_id);
            }
        });
    }
}

async fn on_room_member(
    event: SyncRoomMemberEvent,
    room: Room,
    join_detection_config: &matrix_bot_help::JoinDetectionConfig,
) {
    // Check if join detection is enabled
    if !join_detection_config.enabled {
        return;
    }

    // Only respond to events in joined rooms
    if room.state() != RoomState::Joined {
        return;
    }

    // Check if this room is in the monitored list (if list is not empty)
    if !join_detection_config.monitored_rooms.is_empty() {
        let room_id_str = room.room_id().to_string();
        if !join_detection_config.monitored_rooms.contains(&room_id_str) {
            return;
        }
    }

    // Get the bot's user ID for filtering
    let client = room.client();
    let bot_user_id = client.user_id().expect("Client should have a user ID");

    // Extract the user ID from the event
    let user_id = event.state_key().to_owned();

    // Don't announce when the bot itself joins
    if user_id == bot_user_id {
        return;
    }

    // Check if the user is joining the room
    match event {
        SyncRoomMemberEvent::Original(original) => {
            if original.content.membership == MembershipState::Join {
                // Check if this is actually a new join (not just a state update)
                if let Some(prev_content) = original.prev_content()
                    && prev_content.membership == MembershipState::Join
                {
                    // This is not a new join, just an update to an existing member
                    return;
                }

                println!("User {} joined room {}", user_id, room.room_id());

                // Send welcome message if enabled
                if join_detection_config.send_welcome {
                    // Create a personalized welcome message mentioning the user
                    let welcome_text =
                        format!("{}: {}", user_id, join_detection_config.welcome_message);
                    let response = match join_detection_config.welcome_format {
                        HelpFormat::Plain => RoomMessageEventContent::text_plain(&welcome_text),
                        HelpFormat::Html => {
                            RoomMessageEventContent::text_html(&welcome_text, &welcome_text)
                        }
                        HelpFormat::Markdown => {
                            RoomMessageEventContent::text_markdown(&welcome_text)
                        }
                    };

                    // Send welcome message in the room where the user joined
                    if let Err(e) = room.send(response).await {
                        eprintln!("Failed to send welcome message to {}: {}", user_id, e);
                    } else {
                        println!(
                            "Sent welcome message to {} in room {}",
                            user_id,
                            room.room_id()
                        );
                    }
                }
            }
        }
        SyncRoomMemberEvent::Redacted(_) => {
            // Handle redacted events if needed
        }
    }
}
