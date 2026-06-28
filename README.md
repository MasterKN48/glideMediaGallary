# <img src="static/logo.svg" align="left" width="48" height="48" style="margin-right: 12px;" /> Glide: A Portable Media Gallery App

[![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC107?logo=tauri&logoColor=white&style=flat-square)](https://tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white&style=flat-square)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white&style=flat-square)](https://www.rust-lang.org/)
[![Bun](https://img.shields.io/badge/Bun-v1.0-F9F1E7?logo=bun&logoColor=black&style=flat-square)](https://bun.sh/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

**Glide** is an ultra-fast, lightweight cross-platform desktop media gallery designed to scan and browse massive local directories, SSDs, and HDDs smoothly at 60 FPS in an elegant glassmorphic dark mode. Engineered using a high-performance native **Rust/Tauri** backend and a reactive **Svelte 5** SPA frontend.

> **GitHub Description:** Ultra-fast, lightweight portable media gallery for local and external storage. Built with Tauri, Svelte 5, and Rust. Features parallel SSD scanning, EXIF extraction, custom grid virtualization, and a glassmorphic viewer with metadata panels.

---

## 🚀 Key Features

*   ⚡ **60+ FPS Scroll Performance:** Custom row-reusing virtualized grid handles folders containing tens of thousands of items with zero DOM overhead.
*   🔍 **Instant Reactive Search:** Instantly filters indexed assets by filename as you type, with zero search lag.
*   📸 **Extended EXIF Details:** Toggleable slide-out metadata drawer displaying Device Details (Model, Maker, OS), Camera Settings (Aperture, ISO, Focal Length, Shutter Speed), and GPS Location coordinates.
*   📂 **Show in File Manager:** Open any media file directly in Finder (macOS) or File Explorer (Windows) via a native Tauri command.
*   🔄 **Auto & Manual Orientation:** Extracts EXIF orientation headers to display images correctly and includes a manual `Rotate 90°` control.
*   🎨 **Premium Glassmorphic Dark UI:** Smooth CSS transitions, spring-loaded hover animations, and a sleek web-based glowing startup splash screen.
*   🗄️ **SQLite Local Cache:** Scanned directory indexes are persisted in a local database for lightning-fast subsequent launches (`<50ms`).

---

## 🛠️ Architectural Optimizations

### 1. Parallel Directory Walk (`jwalk` + `rayon`)
The Rust backend traverses directories concurrently using a work-stealing thread pool. Found media items are streamed progressively to Svelte in batches of 100 via Tauri IPC events, allowing you to browse files instantly while scanning is still active in the background.

### 2. Dual-Mode Thumbnail Pipeline
- **EXIF Extraction:** For supported image types, Svelte requests thumbnails which Rust attempts to extract directly from the EXIF segment in microseconds, bypassing full image decoding.
- **Background Decoding:** For other images, Rust decodes and resizes the files to compressed WebP thumbnails in a background worker pool using `tauri::async_runtime::spawn_blocking`, completely keeping the UI thread lag-free.

### 3. DOM Recycle Grid Virtualization
Instead of loading thousands of images into the browser DOM (which causes GPU memory choke), Glide's custom virtual grid calculates the viewport bounds and recycles off-screen rows. Intersection Observers trigger thumbnail requests only when an item enters the viewport.

---

## 📂 Project Structure

```
.
├── src-tauri/                 # Rust Tauri Backend
│   ├── capabilities/          # Security default.json granting Dialog and Window APIs
│   ├── src/
│   │   ├── lib.rs             # Tauri command declarations & app bootstrap
│   │   ├── db.rs              # SQLite schema, indices, and query helpers
│   │   └── scanner.rs         # Parallel file scanner, EXIF reader, & thumbnail engine
│   └── Cargo.toml
├── src/                       # Svelte 5 Frontend
│   ├── lib/
│   │   └── components/
│   │       ├── VirtualGrid.svelte  # Grouped chronological virtualizer
│   │       ├── Thumbnail.svelte    # viewport observer and orientation rotators
│   │       └── MediaPlayer.svelte  # Lightbox with metadata drawers & pan/zoom
│   ├── routes/
│   │   └── +page.svelte       # Sidebar layout controls, search filters, and page skeleton
│   └── app.html
├── static/
│   └── logo.svg               # High-contrast light brand icon
└── package.json
```

---

## 🚀 Getting Started

### Prerequisites
Make sure you have the following installed on your developer machine:
- [Rust & Cargo compiler toolchain](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/) (recommended) or Node.js

### How to Run Locally

1. Clone the repository and navigate to the project directory:
   ```bash
   git clone https://github.com/MasterKN48/glideMediaGallary.git
   cd glideMediaGallary
   ```
2. Install the node dependencies:
   ```bash
   bun install
   ```
3. Run the application in development mode:
   ```bash
   bun run tauri dev
   ```

### Building for Distribution
To build a production-ready, self-contained desktop bundle for the host system:
```bash
bun run tauri build
```
The compiled binaries will be outputted to `src-tauri/target/release/bundle/`.

---

## 📦 Build & Release Guide

### 1. Understanding Release Bundles
*   **`.dmg` (macOS Disk Image):** The standard macOS distribution format. Users double-click to drag the application into their `/Applications` directory.
*   **`.app` (macOS Application):** The raw compiled application bundle directory. Can be launched directly.
*   **`.msi` / `.exe` (Windows):** Windows installer format generated when compiling on a Windows host.
*   **`.deb` / `.AppImage` (Linux):** Debian package and self-contained sandbox execution format generated when compiling on a Linux host.

### 2. Multi-Platform Compilation
Since Tauri builds native binaries, you must compile the final release assets on the respective target operating systems or set up a continuous integration pipeline:
*   **Windows Builds:** Compile on Windows 10/11 or use a Windows VM.
*   **Linux Builds:** Compile on Ubuntu/Debian or run inside a Docker container.
*   **GitHub Actions CI:** You can automate multi-platform builds using a runner workflow matrix that spins up macOS, Windows, and Ubuntu instances concurrently.

### 3. Mobile Support (Android & iOS)
Tauri v2 supports compiling for mobile targets.

#### Prerequisites
- **iOS:** Xcode and Command Line Tools (macOS host only).
- **Android:** Android Studio, SDK Platform Tools, and the Android NDK.

#### Initialization
Add mobile project configurations to your workspace:
```bash
# Add Android configuration wrappers
bun tauri android init

# Add iOS configuration wrappers
bun tauri ios init
```

#### Run & Build
*   **Development Simulator:**
    ```bash
    bun tauri android dev
    # or
    bun tauri ios dev
    ```
*   **Release Compiles:**
    ```bash
    bun tauri android build
    # or
    bun tauri ios build
    ```

---

## 📜 License
Distributed under the MIT License. See [LICENSE](LICENSE) for more details.
