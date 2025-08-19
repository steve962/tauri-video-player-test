use crate::video_player::*;

mod video_player;

#[tauri::command]
async fn open_video_player(url: String, app_handle: tauri::AppHandle) {
    if url.is_empty() {
        return;
    }

    println!("Opening video player with URL: {}", url);
    let player = VideoPlayer::new(url, app_handle.clone());
    PLAYER_COLLECTION.add_player(player);
}

#[tauri::command]
async fn player_event(window: String, event_name: String, data: serde_json::Value) {
    let mut remove: Option<String> = None;

    // Wrap this next section so we don't have two mutable borrows of PLAYER_COLLECTION
    {
        let mut players = PLAYER_COLLECTION.players.lock().unwrap();
        let w = players.get_mut(&window);
        match w {
            Some(player) => { 
                if !player.handle_event(event_name, data) {
                    remove = Some(player.name.clone());
                }
            },
            None => {}
        }
    }

    // If the event handler returns false, it was just closed and should be removed
    if let Some(id) = remove {
        PLAYER_COLLECTION.remove_player(&id);
    }
}

#[tauri::command]
async fn close_all() {
    println!("Closing all video players");
    PLAYER_COLLECTION.close_all();
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_video_player, player_event, close_all])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
