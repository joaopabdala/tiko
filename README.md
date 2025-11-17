# 💾 Tiko: TikTok Video Downloader (CLI)

Tiko is a fast, straight-forward **Command Line Interface (CLI) tool** built with **Rust** for downloading **TikTok videos in HD, without the watermark**.

-----

## ✨ Features

  * **No Watermark:** Downloads the original, high-quality video content.
  * **Simple CLI:** Easy to use directly from your terminal.
  * **Cross-Platform:** Works wherever Rust binaries are supported (Windows, macOS, Linux).
  * **Direct Download:** Requires only the video URL.

-----

## 🚀 Installation & Setup

You just need the compiled binary to run Tiko.

### 1\. Download the Binary

The latest binary can be found on the [Releases page](https://github.com/joaopabdala/tiko/releases).

### 2\. Add to PATH (Recommended)

To run Tiko from any directory without typing `./tiko` every time, move the binary to a directory included in your system's `PATH`.

  * **Linux/macOS:** Move the binary to `/usr/local/bin/` or `~/.local/bin/`.

-----

## 📖 Usage

To download a video, simply execute the `tiko` command followed by the full TikTok video URL.

### Syntax

```bash
tiko <TIKTOK_URL>
```

### Example

The video will be downloaded to your **current working directory** and named based on the username and video ID (e.g., `user012147011_7571521498322652434.mp4`).

```bash
$ tiko https://www.tiktok.com/@user012147011/video/7571521498322652434

📥 Download started for user012147011_7571521498322652434
✅ Download finished 'user012147011_7571521498322652434.mp4'.
```

-----

## 👨‍💻 Development & Motivation

Tiko was created with two main goals:

1.  To serve as a **straightforward, effective solution** for downloading high-quality TikTok videos.
2.  To act as a **hands-on learning project** to master building a robust, production-ready CLI application using **Rust's** asynchronous capabilities.