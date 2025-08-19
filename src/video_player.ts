import { getCurrentWindow } from '@tauri-apps/api/window';
import VideoPlayerHandler from "./video_player_handler.ts";

var player;

window.addEventListener("DOMContentLoaded", () => {
    const appWindow = getCurrentWindow();

    // The window label is the identifier we use for window events to and from Rust.
    if (appWindow.label) {
        console.log("Creating video player handler for window:", appWindow.label);
        player = new VideoPlayerHandler(appWindow.label, appWindow);
    }
});
