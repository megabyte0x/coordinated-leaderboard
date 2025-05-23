use dotenv::dotenv;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::sync::Mutex;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::time::{Duration, interval};

// Global variable for chat ID
static GC_CHAT_ID: Lazy<Mutex<Option<ChatId>>> = Lazy::new(|| Mutex::new(None));

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let pool_arc = Arc::new(pool);

    // Start the leaderboard scheduler in a separate task

    // Handle user commands
    Command::repl(bot, move |bot, msg, cmd| {
        answer(bot, msg, cmd, Arc::clone(&pool_arc))
    })
    .await;

    Ok(())
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Say hello
    #[command(alias = "start")]
    Start,
    /// Display this text.
    #[command(aliases = ["h", "?"])]
    Help,
    /// Show the leaderboard.
    #[command(alias = "lb")]
    Leaderboard,
}

async fn answer(bot: Bot, msg: Message, cmd: Command, pool: Arc<PgPool>) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            // Update the global chat ID
            let chat_id = msg.chat.id;
            {
                let mut gc_chat_id = GC_CHAT_ID.lock().unwrap();
                *gc_chat_id = Some(chat_id);
            }

            let bot_clone = bot.clone();
            let pool_clone = pool.clone();

            tokio::spawn(async move {
                run_leaderboard_scheduler(bot_clone, pool_clone).await;
            });

            bot.send_message(msg.chat.id, "Hello, world! Chat ID has been saved.")
                .await?
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Leaderboard => {
            let leaderboard = get_leaderboard(&pool).await;

            bot.send_message(msg.chat.id, leaderboard).await?
        }
    };

    Ok(())
}

// Scheduler function to send leaderboard periodically
async fn run_leaderboard_scheduler(bot: Bot, pool: Arc<PgPool>) {
    // Create an interval that fires every day
    let mut interval = interval(Duration::from_secs(86400));
    let target_chat_id = GC_CHAT_ID.lock().unwrap().unwrap();

    loop {
        // Wait until the next tick
        interval.tick().await;

        // Get the current leaderboard
        let leaderboard = get_leaderboard(&pool).await;

        // Send the leaderboard message
        if let Err(e) = bot.send_message(target_chat_id, leaderboard).await {
            log::error!("Failed to send scheduled leaderboard: {:?}", e);
        } else {
            log::info!("Sent scheduled leaderboard");
        }
    }
}

async fn format_leaderboard(recs: Vec<LeaderBoard>) -> String {
    let mut leaderboard_text = vec!["Leaderboard 🏆".to_string()];

    leaderboard_text.extend(recs.iter().enumerate().map(|(index, rec)| {
        let username = format!("@{}", rec.telegram_username);
        format!("{}. {:<20} {}", index + 1, username, rec.xp)
    }));

    leaderboard_text.join("\n")
}

async fn get_leaderboard(pool: &PgPool) -> String {
    let recs = sqlx::query_as::<_, LeaderBoard>(
        "SELECT telegram_username, xp FROM leaderboard ORDER BY xp DESC",
    )
    .fetch_all(pool)
    .await
    .unwrap();

    format_leaderboard(recs).await
}

#[derive(sqlx::FromRow)]
struct LeaderBoard {
    telegram_username: String,
    xp: i32,
}
