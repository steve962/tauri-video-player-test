import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { TauriEvent } from "@tauri-apps/api/event";

export default class VideoPlayerHandler
{
    interval: number | null;
    name: string;
    unlistens: UnlistenFn[];
    appWindow: any;

    constructor(name: string, window: any) {
        this.interval = null;
        this.name = name;
        this.unlistens = [];
        this.appWindow = window;

        this.attach();
        this.send_event("ready", {});
    }

    attach() {
        let _self = this;

        document.querySelector("#video-play")?.addEventListener("click", (e) => {
            e.preventDefault();
            _self.send_event("play", {});
        });

        document.querySelector("#video-pause")?.addEventListener("click", (e) => {
            e.preventDefault();
            _self.send_event("pause", {});
        });

        // Poll periodically for errors on the video player
        if (!_self.interval) {
            _self.interval = setInterval(() => {
                _self.send_event("poll", {});
            }, 500);
        }

        // Listen for events from the Rust backend
        listen<any>(self.name, (params) => {
            _self.onWindowEvent(params.payload.event, params.payload.data);
        }).then((unlisten: UnlistenFn) => {
            _self.unlistens.push(unlisten);
        });

        // Listen for window resize, move, and close events
        _self.appWindow.listen(TauriEvent.WINDOW_RESIZED, ({ payload }: { payload: any }) => {
            _self.onWindowResized(payload);
         }).then((unlisten: UnlistenFn) => {
            _self.unlistens.push(unlisten);
        });

        _self.appWindow.listen(TauriEvent.WINDOW_MOVED, ({ payload }: { payload: any }) => {
            _self.onWindowMoved(payload);
         }).then((unlisten: UnlistenFn) => {
            _self.unlistens.push(unlisten);
        });

        _self.appWindow.listen(TauriEvent.WINDOW_CLOSE_REQUESTED, () => {
            _self.close();
         }).then((unlisten: UnlistenFn) => {
            _self.unlistens.push(unlisten);
        });
    }

    // Clean up resources and detach event listeners
    detach() {
        let _self = this;

        if (_self.interval) {
            clearInterval(_self.interval);
            _self.interval = null;
        }

        var unlisten: UnlistenFn | undefined;
        while (unlisten = _self.unlistens.pop()) {
            unlisten();
        }
    }

    // Send the closed event, clean up, and close the video player window
    close() {
        let _self = this;

        _self.send_event("closed", {});
        _self.detach();
        _self.appWindow.close();
    }

    // Send an event to the Rust backend with the specified name and data
    async send_event(name: string, eventData: any) {
        let _self = this;
        console.log(`Sending event: ${name} with data:`, eventData);
        invoke("player_event", {window: _self.name, eventName: name, data: eventData });
    }


    // Handle events from the Rust backend.  Mainly used to handle the close event when
    // the main window is closed.
    onWindowEvent(event: string, _params: any) {
        let _self = this;
        console.log(`onWindowEvent: ${event} for player ${_self.name}`);
        if (event === "close") {
            _self.detach();
            _self.appWindow.close();
        }
    }

    // Handle window resize events
    onWindowResized(params: any) {
        let _self = this;
        console.log("onWindowResized for player:", _self.name, ", parameters: ", params);
        _self.send_event("resized", params);
    }

    // Handle window moved events
    onWindowMoved(params: any) {
        let _self = this;
        console.log("onWindowMoved for player:", _self.name, ", parameters: ", params);
        _self.send_event("moved", params);
    }


}