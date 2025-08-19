import { getCurrentWindow } from '@tauri-apps/api/window';
import VideoPlayerHandler from "./video_player_handler.ts";

var player;

window.addEventListener("DOMContentLoaded", () => {
    const appWindow = getCurrentWindow();

    if (appWindow.label) {
        console.log("Creating video player handler for window:", appWindow.label);
        player = new VideoPlayerHandler(appWindow.label, appWindow);
    }
});
