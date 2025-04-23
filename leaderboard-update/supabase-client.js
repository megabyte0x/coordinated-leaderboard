import { createClient } from '@supabase/supabase-js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

// Initialize Supabase client
const supabaseUrl = process.env.SUPABASE_URL;
// Use the service role key instead of the anon key for admin operations
const supabaseKey = process.env.SUPABASE_ANON_KEY;

if (!supabaseUrl || !supabaseKey) {
    throw new Error('Supabase URL and service role key must be provided in environment variables');
}

const supabase = createClient(supabaseUrl, supabaseKey);

/**
 * Updates or inserts a user in the leaderboard table
 * @param {string} telegramUsername - User's Telegram username 
 * @param {number} xp - User's XP points
 * @param {string} [xHandle] - Optional X (Twitter) handle
 * @returns {Promise<object>} - Result of the operation
 */
export async function updateLeaderboard(telegramUsername, xp, xHandle = null) {
    try {
        // Check if the user exists
        const { data: existingUser } = await supabase
            .from('leaderboard')
            .select('telegram_username, xp')
            .eq('telegram_username', telegramUsername)
            .single();

        // Prepare the data object based on whether xHandle is provided
        const userData = xHandle
            ? { xp, x_handle: xHandle }
            : { xp };

        if (existingUser) {
            // Update existing user
            const { data, error } = await supabase
                .from('leaderboard')
                .update(userData)
                .eq('telegram_username', telegramUsername)
                .select();

            if (error) throw error;
            return { success: true, data, operation: 'update' };
        }

        // Insert new user (no else needed, previous branch returns early)
        const insertData = {
            telegram_username: telegramUsername,
            xp,
            ...(xHandle && { x_handle: xHandle })
        };

        const { data, error } = await supabase
            .from('leaderboard')
            .insert([insertData])
            .select();

        if (error) throw error;
        return { success: true, data, operation: 'insert' };
    } catch (error) {
        console.error('Error updating leaderboard:', error);
        return { success: false, error: error.message };
    }
}

/**
 * Gets the current leaderboard, sorted by XP in descending order
 * @returns {Promise<Array>} - Sorted leaderboard data
 */
export async function getLeaderboard() {
    try {
        const { data, error } = await supabase
            .from('leaderboard')
            .select('telegram_username, x_handle, xp')
            .order('xp', { ascending: false });

        if (error) throw error;
        return data;
    } catch (error) {
        console.error('Error fetching leaderboard:', error);
        return [];
    }
} 