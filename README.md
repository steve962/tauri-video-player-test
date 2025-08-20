# Tauri Video Player Test

This is a test program showing how to play a video stream in Tauri using the GStreamer rust bindings.

I'm putting the source up for two reasons:

1) As a reference for getting assistance with some known issues

2) As a stepping point for other people trying to figure out how to work with Rust, GStreamer, and Tauri and trying to put them all together since there doesn't seem to be a good starting point for this yet.

This app is based on the standard Tauri template for Vanilla Typescript using NPM.


## Requirements

You will need all the typical Rust Tauri development environment tools, including:

- Git
- NPM
- Cargo
- VSCode or some other compiler environment
- If building on Windows, you'll need the MSVC 2019 Toolset.

In addition, you will need to have GStreamer binaries installed as per the GStreamer Rust library page here:

```
    https://docs.rs/gstreamer/latest/gstreamer/
```    

## Setting up and running

Clone this project to your local machine, and from a terminal or Powershell window (depending on platform), run:

```
    npm install
    npm run tauri dev
```

Once the main window is up and running, simply type in or paste a URI to a streaming video source and click on the button to play the video in a secondary window which will open.   You can even have multiple videos playing at the same time.

Most of my testing has been with RTSP streams since that's my primary application.  I've also tested a few MP4 videos on the web successfully.

I haven't tested this with local video files, but gstreamer ought to be able to handle those as well using a file:// style URI.


## Platform Notes

### Linux (Debian 12)

On my Debian 12 system, this runs fine, and plays most RTSP streams I've given it quite well, plus some MP4s I've tested.   That said, I've had no luck with HLS or WebRTC (There are known issues with WebRTC in gstreamer although they're working on it...) streams and I haven't tried an RTMP or SRT stream yet.   I did have Gnome Desktop crash and restart when I tried closing the window (shutting down the pipeline) after attempting to display the Wowza demo stream referenced on this page:
```
    https://www.wowza.com/developer/rtsp-stream-test
```

I don't know why that happened, and debugging is difficult since it shut everything down and restarted, but based on other tests the stream failed due to the URI returning a 403 error.


### Mac

The first time I tried this on a Mac, it built, ran, and took a long time before displaying the first video.   The app itself crashed a minute or so later.   After that first time, I've been able to run it pretty consistently and had similar results to Linux, with no further crashes.   Logs from the initial run showed a lot of GStreamer configuration messages, so maybe it was a one time setup thing?   Anyway - it seems to work but might need some investigation.


### Windows

By using a bit of a hack, I was able to get video to work on Windows as well.   The problem here is that when we set the GStreamer VideoOverlay to
the tauri WebviewWindow handle, the actual video appears *behind* the webview instead of in the front of it like Linux and Mac.   When I first ran this, I thought video wasn't working at all... until I resized rapidly and caught flickers of video behind the webview.

There is a code workaround for this, which involves making the Webview invisible when the video is playing, and then visible again when it stops.   The major downside of this is that you cannot put controls on the Webview and display the video in

While the program runs, I was unable to get visible video to work.   That doesn't mean it's not working, however...  what I discovered here was that the window webview contents were sitting ABOVE the video.   Resizing rapidly gets yuo flickers of video.   This is the issue I'm trying to resolve by posting this code, so hopefully I'll have a fix in before many people look at it.


