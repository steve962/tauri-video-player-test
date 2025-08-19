use raw_window_handle::{RawWindowHandle, HasWindowHandle};
use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder, WebviewUrl, Emitter };
use gstreamer::prelude::*;
use gstreamer_video::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::LazyLock;
use inflector::cases::snakecase::to_snake_case;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct VideoPlayer {
    pub name: String,
    pub app_handle: AppHandle,
    pub window: Option<WebviewWindow>,
    pub width: f64,
    pub height: f64,
    pub url: String,
    pub pipeline: Option<gstreamer::Element>,
    pub video_overlay: Option<gstreamer_video::VideoOverlay>,   
}

impl VideoPlayer {
    pub fn new(url: String, app_handle: AppHandle) -> Self {
        Self {
            name: "player-".to_string() + &to_snake_case(&url),
            app_handle,
            window: None,
            width: 800.0,
            height: 600.0,
            url: url,
            pipeline: None,
            video_overlay: None,
        }
    }

    pub fn build_pipeline_for_url(&mut self, window: &Option<WebviewWindow>, url: &str) {
        println!("Building pipeline for URL: {}", url);
        gstreamer::init().unwrap();

        let sink = gstreamer::ElementFactory::make("glimagesink")
            .name("sink")
            .build()
            .expect("Could not create sink element");

        let pipeline = gstreamer::parse::launch(&format!("playbin uri={}", url))
            .expect("Could not create pipeline from launch string");
        pipeline.set_property("video_sink", &sink);

        let video_overlay = match sink.dynamic_cast::<gstreamer_video::VideoOverlay>() {
            Ok(overlay) => overlay,
            Err(_) => {
                println!("Could not get VideoOverlay from sink element");
                return;
            }
        };

        match window {
            Some(window) => {

                match window.window_handle().unwrap().as_raw() {
                    RawWindowHandle::UiKit(handle) => {
                        unsafe {
                            video_overlay.set_window_handle(handle.ui_view.as_ptr().expose_provenance());
                        }
                    }
                    RawWindowHandle::AppKit(handle) => {
                        unsafe {
                            video_overlay.set_window_handle(handle.ns_view.as_ptr().expose_provenance());
                        }
                    }
                    RawWindowHandle::Xlib(handle) => {
                        unsafe {
                            video_overlay.set_window_handle(handle.window.try_into().unwrap());
                        }
                    }
                    RawWindowHandle::Win32(handle) => {
                        unsafe {
                            video_overlay.set_window_handle(handle.hwnd.get() as _);
                        }
                    },
                    _ => {
                        println!("VideoStreamer: No window handle set for gstreamer element");
                        return;
                    }
                }
            },
            None => {
                println!("VideoStreamer: No window set for gstreamer element");
                return;
            },
        }

        let _ = video_overlay.set_render_rectangle(0, 20, 300, 200);

        self.pipeline = Some(pipeline);
        self.video_overlay = Some(video_overlay);
    }

    pub fn change_video_size(&self, x: i32, y: i32, width: i32, height: i32) {
        println!("change_video_size called with x: {}, y: {}, width: {}, height: {}", x, y, width, height);
        if let Some(overlay) = &self.video_overlay {
            let _ = overlay.set_render_rectangle(x, y, width, height);
            overlay.expose();
        }
    }

    pub fn fill_current_window(&self) {
        self.change_video_size(0, 0, self.width as i32, self.height as i32);
//        self.change_video_size(2, 2, self.width as i32 - 4, self.height as i32 - 4);
    }

    pub fn play(&self) {
        println!("Playing player: {}", self.name);
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gstreamer::State::Playing)
                .expect("Unable to set the pipeline to the `Playing` state");
        }
    }

    pub fn pause(&self) {
        println!("Pausing player: {}", self.name);
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gstreamer::State::Paused)
                .expect("Unable to set the pipeline to the `Paused` state");
        }
    }

    pub fn stop(&mut self) {
        println!("Stopping player: {}", self.name);
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(gstreamer::State::Null)
                .expect("Unable to set the pipeline to the `Null` state");
        }
        self.pipeline = None;
        self.video_overlay = None;
    }

    /// Polls the pipeline for errors or end of stream messages so we can shut down gracefully.
    /// This is currently being called through the a Javascript interval from the front end
    /// but you might want to do this in a service thread
    pub fn poll(&mut self) -> bool {
        if let Some(pipeline) = &self.pipeline {
            // Check for error or EOS
            let bus = pipeline.bus().unwrap();
            while let Some(msg) = bus.pop() {
                use gstreamer::MessageView;

                match msg.view() {
                    MessageView::Error(err) => {
                        eprintln!(
                            "Error received from element {:?}: {}",
                            err.src().map(|s| s.path_string()),
                            err.error()
                        );
                        eprintln!("Debugging information: {:?}", err.debug());
                        self.stop();
                        return false;
                    }
                    MessageView::Eos(..) => {
                        self.stop();
                        return false;
                    },
                    _ => (),
                }
            }
        }
        return true;
    }

    pub fn open(&mut self) -> tauri::Result<()> {
        println!("Opening player: {}", self.name);
        self.width = 404.0;
        self.height = 324.0;
        if self.window.is_none() {
            let window = WebviewWindowBuilder::new(&self.app_handle, &self.name, WebviewUrl::App("video_player.html".to_string().into()))
                .inner_size(self.width, self.height)
//                .decorations(false)
                .title(&self.url)
//                .parent(&self.app_handle.get_webview_window(&self.parent).unwrap())?
                .build()?;
            self.window = Some(window);
        }
        Ok(())
    }

    pub fn close(&mut self) {
        println!("Closing player: {}", self.name);
        self.stop();
        if let Some(window) = &self.window {
            window.close().unwrap();
        }
        self.window = None;
    }

    pub fn send_event(&self, event_name: String, data: serde_json::Value) {
        let payload = json!({
            "event": event_name,
            "data": data, 
        });
        println!("Sending event: {} with data: {:?} on player {}", event_name, data, self.name);
        self.app_handle.emit(&self.name, payload).unwrap();
    }

    pub fn handle_event(&mut self, event_name: String, data: serde_json::Value) -> bool {
        match event_name.as_str() {
            "ready" => {
                if let Some(window) = &self.window {
                    self.build_pipeline_for_url(&Some(window.clone()), &self.url.clone());
                    self.fill_current_window();
                    self.play();
                }
            },
            "poll" => {
                if !self.poll() {
                    self.send_event("closed".to_string(), serde_json::Value::Null);
                }
            }
            "play" => self.play(),
            "pause" => self.pause(),
            "stop" => self.stop(),
            "resized" => {
                if let Some(width) = data.get("width").and_then(|v| v.as_f64()) {
                    if let Some(height) = data.get("height").and_then(|v| v.as_f64()) {
                        self.width = width;
                        self.height = height;
                        self.fill_current_window();
                    }
                }
            },
            "closed" => {
                self.close();
                return false; // Indicate that the player should be removed
            },
            _ => println!("Unhandled event: {}", event_name),
        }
        return true; // Indicate that the player is still active
    }

}

/// This is a way to keep track of all the open video players in a thread-safe manner.
pub struct VideoPlayerCollection {
    pub players: Arc<Mutex<HashMap<String, VideoPlayer>>>,
}

impl VideoPlayerCollection {
    pub fn new() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn lock_players(&self) -> std::sync::MutexGuard<HashMap<String, VideoPlayer>> {
        self.players.lock().unwrap()
    }

    /// Adds a new player to the collection and opens it.
    /// If the player already exists, it will return false because we can't have two players with the same window name
    pub fn add_player(&self, player: VideoPlayer) -> bool {
        let mut players = self.lock_players();
        let name = player.name.clone();
        if players.contains_key(&name) {
            println!("Player with name {} already exists", name);
            return false;
        }
        players.insert(player.name.clone(), player);
        let pl = players.get_mut(&name);
        if let Some(p) = pl {
            p.open().unwrap();
        }
        return true;
    }

    pub fn remove_player(&self, id: &str) {
        let mut players = self.lock_players();
        players.remove(id);
    }

    pub fn close_all(&self) {
        println!("Closing all video players");
        let mut players = self.lock_players();
        for player in players.values_mut() {
            player.send_event("close".to_string(), serde_json::Value::Null);
            player.close();
        }
    }
    
}

/// For simplicity, we're creating a static instance of the VideoPlayerCollection.
/// In a real application, you might want to manage this differently.
pub static PLAYER_COLLECTION: LazyLock<VideoPlayerCollection> = LazyLock::new(|| {
    VideoPlayerCollection::new()
});
