use crate::game_config_util::{ServerSettings};

pub async fn get_game_config_by_user_id(user_id: i32) -> anyhow::Result<ServerSettings> {
    let settings_data = ServerSettings {
        server_name: "Local Game Host".to_string(),
        server_description: "A 7 Days to Die server".to_string(),
        server_password: "".to_string(),
        language: "English".to_string(),
        server_max_player_count: 8,
        eac_enabled: false,
        game_difficulty: 1,
        party_shared_kill_range: 100,
        player_killing_mode: 3
    };
    Ok(settings_data)
}