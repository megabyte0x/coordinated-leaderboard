use dotenv::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::time::{Duration, interval};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();
    let bot_clone = bot.clone();

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let pool_arc = Arc::new(pool);
    let pool_clone = Arc::clone(&pool_arc);

    // Start the leaderboard scheduler in a separate task
    tokio::spawn(async move {
        run_leaderboard_scheduler(bot_clone, pool_clone).await;
    });

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
    /// Register with your x-handle. Example: /register username
    #[command(alias = "r")]
    Register { handle: String },
}

async fn answer(bot: Bot, msg: Message, cmd: Command, pool: Arc<PgPool>) -> ResponseResult<()> {
    match cmd {
        Command::Start => bot.send_message(msg.chat.id, "Hello, world!").await?,
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Leaderboard => {
            let leaderboard = get_leaderboard(&pool).await;

            bot.send_message(msg.chat.id, leaderboard).await?
        }
        Command::Register { handle } => {
            let username = msg.from.unwrap().username;

            match username {
                Some(username) => {
                    if handle.is_empty() {
                        bot.send_message(msg.chat.id, "Please provide your X Handle")
                            .await?
                    } else {
                        register_user(&pool, username, &handle).await.unwrap();
                        bot.send_message(
                            msg.chat.id,
                            format!("Registered with X Handle {}.", handle),
                        )
                        .await?
                    }
                }
                None => {
                    bot.send_message(msg.chat.id, format!("Something went wrong"))
                        .await?
                }
            }
        }
    };

    Ok(())
}

// Scheduler function to send leaderboard periodically
async fn run_leaderboard_scheduler(bot: Bot, pool: Arc<PgPool>) {
    let target_chat_id = ChatId(std::env::var("GC_CHAT_ID").unwrap().parse::<i64>().unwrap());

    // Create an interval that fires every 10 seconds
    let mut interval = interval(Duration::from_secs(86400));

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
        "SELECT telegram_username, xp FROM leaderboard ORDER BY xp DESC LIMIT 5",
    )
    .fetch_all(pool)
    .await
    .unwrap();

    format_leaderboard(recs).await
}

async fn register_user(
    pool: &PgPool,
    tg_username: String,
    x_handle: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO leaderboard (telegram_username, x_handle, xp) 
         VALUES ($1, $2, 0) 
         ON CONFLICT (telegram_username) DO UPDATE 
         SET x_handle = $2",
    )
    .bind(tg_username)
    .bind(x_handle)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct LeaderBoard {
    telegram_username: String,
    xp: i32,
}
