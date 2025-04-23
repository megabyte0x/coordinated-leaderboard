# Supabase Leaderboard Integration

A simple JavaScript client for interacting with a Supabase leaderboard table, designed to work with the coordinated-leaderboard project.

## Setup

1. Install dependencies:
   ```
   npm install
   ```

2. Create a `.env` file based on `supabase.env.example`:
   ```
   cp supabase.env.example .env
   ```

3. Update the `.env` file with your Supabase credentials:
   - `SUPABASE_URL`: Your Supabase project URL
   - `SUPABASE_ANON_KEY`: Your Supabase anonymous key

## Database Setup

Your Supabase database should have a `leaderboard` table with the following schema:

```sql
CREATE TABLE leaderboard (
  id SERIAL PRIMARY KEY,
  telegram_username TEXT UNIQUE NOT NULL,
  x_handle TEXT,
  xp INTEGER NOT NULL DEFAULT 0
);
```

## Usage

The `supabase-client.js` file exports the following functions:

### updateLeaderboard(telegramUsername, xp, xHandle)

Updates or inserts a user record in the leaderboard table.

- `telegramUsername` (required): The Telegram username
- `xp` (required): Experience points for the user
- `xHandle` (optional): The user's X (Twitter) handle

### getLeaderboard()

Returns the current leaderboard sorted by XP in descending order.

## Data Format

The example uses a JSON file `leaderboard-data.json` to store user data:

```json
[
  {
    "telegram_username": "user1",
    "xp": 100
  },
  {
    "telegram_username": "user2",
    "xp": 150,
    "x_handle": "twitter_user2"
  }
]
```

You can modify this file to add or update users in your own implementation.

## Example

See `update-leaderboard-example.js` for a complete usage example. Run it with:

```
npm start
```
