import fs from 'node:fs/promises';
import { updateLeaderboard, getLeaderboard } from './supabase-client.js';



async function main() {
    try {
        // Read data from JSON file
        const rawData = await fs.readFile('./leaderboard-data.json', 'utf-8');
        const users = JSON.parse(rawData);

        // Process each user from the JSON file
        for (const user of users) {
            const { telegram_username, xp, x_handle } = user;

            console.log(`Processing user: ${telegram_username} with XP: ${xp}${x_handle ? `, X: ${x_handle}` : ''}`);

            // Update the leaderboard for this user
            const result = await updateLeaderboard(telegram_username, xp, x_handle || null);

            if (result.success) {
                console.log(`Operation successful: ${result.operation}`);
                console.log(result.data);
            } else {
                console.error(`Operation failed for ${telegram_username}:`, result.error);
            }

            console.log('---');
        }

        // Get and display the current leaderboard
        console.log('\nCurrent Leaderboard:');
        const leaderboard = await getLeaderboard();
        leaderboard.forEach((entry, index) => {
            const xHandleDisplay = entry.x_handle ? ` (${entry.x_handle})` : '';
            console.log(`${index + 1}. @${entry.telegram_username}${xHandleDisplay}: ${entry.xp} XP`);
        });
    } catch (error) {
        console.error('Error in main function:', error);
    }
}

main(); 