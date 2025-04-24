# Coordinated Leaderboard

This is a Telegram bot that allows you to track the leaderboard of the Coordinated Eigen Layer Cohort.

## Setup

1. Clone the repository
2. `cd bot`
3. Create a `.env` file
4. Add the following variables:
    - `TELOXIDE_TOKEN`: The token for the Telegram bot
    - `DATABASE_URL`: The URL for the database
5. Run `cargo run`
6. Chat with the bot on Telegram: `@<YOUR_BOT_USERNAME>`

## Commands

- `/register <x-handle>`: Register your X handle with the bot
- `/leaderboard`: Show the leaderboard
- `/help`: Show the help message
