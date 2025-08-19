import { invoke } from "@tauri-apps/api/core";
import { UnlistenFn } from '@tauri-apps/api/event';
import { TauriEvent } from "@tauri-apps/api/event";
import { getCurrentWindow } from '@tauri-apps/api/window';

let videoInputEl: HTMLInputElement | null;
let unlisten: UnlistenFn | null;

async function open_video() {
  console.log("open_video called");
  if (videoInputEl) {
    console.log("Opening video player with URL:", videoInputEl.value);
    await invoke("open_video_player", {
      url: videoInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  videoInputEl = document.querySelector("#video-input");

  /* Listen for the close event on the main window and trigger closing all the video players */
  let appWindow = getCurrentWindow();
  appWindow.listen(TauriEvent.WINDOW_CLOSE_REQUESTED, () => {
    invoke("close_all", {});
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    appWindow.close().then(() => { console.log("Main Window closed"); });
  }).then((unlistenFn: UnlistenFn) => {
    unlisten = unlistenFn;
  });

  /* Open a video player with the URL in the input field when the form is submitted */
  document.querySelector("#video-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    open_video();
  });
});
