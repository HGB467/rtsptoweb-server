# RTSP to Web (Rust Server)

A high-performance Rust server that streams **RTSP** video to the **web** via **WebRTC** and **HLS**, using **GStreamer Rust bindings**.  
This server provides the backend infrastructure for browser-based live streaming of RTSP sources.

---

## ✨ Features

- **RTSP input** supporting both **H.264** and **H.265** video streams.
- **No audio support** (only video is streamed).
- **WebRTC** streaming using **GStreamer WebRTC** plugin.
- **HLS** streaming available for rewinding and broader device compatibility.
- **Flexible encoding options** for both WebRTC and HLS:
    - **No Re-encode**: Stream directly without re-encoding.
    - **Single Re-encode**: Re-encode once to a specified quality.
    - **Adaptive Re-encode**:
        - For **WebRTC**, adaptive bitrate and resolution based on user's network conditions using Google's **Congestion Control Algorithm**.
        - For **HLS**, streams are re-encoded at **480p**, **720p**, and **1080p** qualities for adaptive delivery.

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [GStreamer](https://gstreamer.freedesktop.org/) and essential plugins

#### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Install GStreamer

**Linux (Ubuntu/Debian)**

```bash
sudo apt update
sudo apt install -y gstreamer1.0-tools gstreamer1.0-plugins-base \
gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-webrtc
```

**macOS (using Homebrew)**

```bash
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav
brew install gst-plugins-rs
```

**Windows**

- Download the [GStreamer MSVC development installer](https://gstreamer.freedesktop.org/download/) (recommended over MinGW version).
- Install both **runtime** and **development** packages.
- Add GStreamer's `bin` directory to your system's **PATH** environment variable.

Example (default path):

```
C:\gstreamer\1.0\msvc_x86_64\bin
```

---

### Building the Server

Clone the repository:

```bash
git clone https://github.com/your-org/rtsptoweb-server.git
cd rtsptoweb-server
```

Build the project:

```bash
cargo build --release
```

---

### Running the Server

```bash
cargo run --release
```

This will start the server and prepare it to accept WebRTC and HLS connections.

---

## 📦 Project Structure

- **RTSP ingestion**: Pulls RTSP streams from cameras or external servers.
- **WebRTC output**: Provides low-latency live streaming using GStreamer's WebRTC implementation.
- **HLS output**: Offers adaptive bitrate streaming (480p, 720p, 1080p) suitable for rewind and compatibility.
- **Encoding options**: Choose between direct passthrough, single re-encode, or adaptive re-encoding based on network conditions.

---

## 🔗 Additional Notes

- **Audio streaming is not supported** — only video streams are handled.
- **Adaptive streaming** ensures users with slower networks still get smooth playback through bitrate adjustments.

