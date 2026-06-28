# Glide: A Portable Media Gallery App

Glide is an ultra-fast, lightweight cross-platform media gallery designed to scan and browse large local and external SSDs/HDDs smoothly at 60 FPS in an elegant glassmorphic dark mode. Built using a native **Rust/Tauri** backend and a **Svelte 5 (Vanilla JavaScript)** frontend.

---

## ✨ Features & Architectural Optimizations

To handle folders with tens of thousands of media files without UI stutter or heavy disk loading, the application implements the following optimizations:

### 1. Zero-DOM-Overhead Svelte 5 Frontend
- Uses Svelte 5's compiled reactivity (Runes: `$state`, `$derived`, `$derived.by`) which updates the DOM surgically.
- Avoids the runtime virtual DOM reconciliation overhead of React, saving CPU and memory.
- Written entirely in vanilla JavaScript (no TypeScript compilation lag).

### 2. Custom Row-Based Grid Virtualization (60+ FPS Scrolling)
- Implements a custom virtual grid wrapper. Instead of rendering thousands of image tags into the DOM (which leads to GPU choke and UI lagging), it dynamically calculates which rows are visible in the viewport and only renders those.
- As the user scrolls, off-screen DOM nodes are recycled.
- Employs `IntersectionObserver` to lazy-load media thumbnails only when they enter the scroll viewport.

### 3. Parallel Directory Scanner (`jwalk` + `rayon`)
- The Rust backend performs parallel directory traversal using a work-stealing thread pool, enabling high-speed directory walking on fast SSDs.
- Streams files to Svelte progressively in batches of 100 items via Tauri IPC. The user starts seeing photos instantly as soon as the scan starts.

### 4. Dual-Mode Thumbnail Pipeline
- **EXIF Extraction:** For photos, Rust reads the EXIF header to extract embedded pre-rendered thumbnails in micro-seconds, bypassing full image decoding.
- **Background Generation:** For images without EXIF thumbnails, Rust generates a compressed WebP/JPEG thumbnail in a background worker thread pool using the `image` crate upon Svelte viewport intersection.
- **SQLite DB Caching:** Scanned folder structures, file metadata, and cached thumbnail paths are stored in a local SQLite database (`rusqlite`). Subsequent app starts load instantly in `<50ms`.

### 5. Premium Glassmorphic Dark Theme
- Styled entirely in Vanilla CSS using translucent colors, border gradients, and backdrop filters for a modern macOS glassmorphic aesthetic.
- Includes a full-screen media lightbox supporting mouse zoom/pan, HTML5 video player, audio playback card, and responsive keyboard navigation (Esc to close, Left/Right arrow keys to browse, Space to play/pause video).

---

## 📂 Project Structure

- **`src-tauri/` (Rust Backend)**
  - [`src/lib.rs`](src-tauri/src/lib.rs): Main entry point, command router, and plugin register.
  - [`src/db.rs`](src-tauri/src/db.rs): SQLite database schema configuration and read/write helpers.
  - [`src/scanner.rs`](src-tauri/src/scanner.rs): Parallel directory walk, EXIF parsing, and background thumbnail resizing.
  - [`tauri.conf.json`](src-tauri/tauri.conf.json): Custom security settings, CSP configuration, and `asset://` local file protocol enabling.
- **`src/` (Svelte JS Frontend)**
  - [`routes/+page.svelte`](src/routes/+page.svelte): Layout skeleton, sidebar directories list, type filter tabs, and glassmorphic styles.
  - [`lib/components/VirtualGrid.svelte`](src/lib/components/VirtualGrid.svelte): Grouping timeline (Day/Month/Year) and row-reversing grid virtualizer.
  - [`lib/components/Thumbnail.svelte`](src/lib/components/Thumbnail.svelte): Viewport observer and dynamic thumbnail load trigger.
  - [`lib/components/MediaPlayer.svelte`](src/lib/components/MediaPlayer.svelte): Image/Video/Audio lightbox viewer with pan/zoom and index browsing.

---

## 🚀 Getting Started

### Prerequisites
Ensure you have the following installed on your system:
- **Bun** (recommended frontend runtime) or **Node.js**
- **Rust** (Cargo compiler toolchain)

### How to Run Locally
1. Clone the repository and navigate to the project directory:
   ```bash
   cd MyMediaGallaryApp
   ```
2. Install the node/bun dependencies:
   ```bash
   bun install
   ```
3. Launch the desktop application in development mode:
   ```bash
   bun run tauri dev
   ```

### Building for Distribution
To build a production-ready, self-contained desktop bundle (`.app` for macOS, `.exe` for Windows, `.deb` for Linux):
```bash
bun run tauri build
```
The compiled binaries will be outputted to `src-tauri/target/release/bundle/`.
