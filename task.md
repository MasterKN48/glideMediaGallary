# Task List - Media Gallery App

## Phase 1: Environment Setup & Scaffolding
- [x] Verify Rust path and shell environment `rustc`/`cargo` availability <!-- id: 1.1 -->
- [x] Scaffold Tauri + Svelte JS application using Bun <!-- id: 1.2 -->
- [x] Install required Bun frontend dependencies <!-- id: 1.3 -->
- [x] Run the skeleton application (`bun run tauri dev`) to verify setup <!-- id: 1.4 -->

## Phase 2: Rust Backend (Disk I/O & Caching)
- [x] Add backend Rust dependencies (`jwalk`, `rayon`, `rusqlite`, `serde`, `tokio`) <!-- id: 2.1 -->
- [x] Implement parallel directory scanner in Rust using `jwalk` <!-- id: 2.2 -->
- [x] Setup local SQLite database schema for caching scanned metadata <!-- id: 2.3 -->
- [x] Create Rust commands to add, remove, and retrieve scanned directories <!-- id: 2.4 -->
- [x] Implement progressive streaming emitter for directory scans via Tauri events <!-- id: 2.5 -->

## Phase 3: Tauri Custom Protocol & EXIF Extraction
- [x] Register custom Tauri asset protocol (`asset://`) for safe local media streaming <!-- id: 3.1 -->
- [x] Implement fast EXIF extraction in Rust for JPEGs/RAWs/HEICs <!-- id: 3.2 -->
- [x] Create background thumbnail generator worker for resizing other files <!-- id: 3.3 -->

## Phase 4: Svelte Frontend & Glassmorphism UI
- [x] Implement vanilla CSS styling foundation for dark glassmorphic UI <!-- id: 4.1 -->
- [x] Build Svelte layout structure (translucent sidebar, timeline view switcher) <!-- id: 4.2 -->
- [x] Create media player modals (image viewer with zoom, HTML5 video player, audio panel) <!-- id: 4.3 -->

## Phase 5: Virtualized Grid & Timeline Rendering
- [x] Write date-grouping (Day/Month/Year) Svelte logic <!-- id: 5.1 -->
- [x] Implement custom virtualized grid in Svelte for high FPS scrolling <!-- id: 5.2 -->
- [x] Integrate Svelte viewport lazy-loading directive (`IntersectionObserver`) <!-- id: 5.3 -->

## Phase 6: Integration, OS Tuning & Polish
- [x] Connect Svelte frontend events to Tauri IPC rust commands <!-- id: 6.1 -->
- [x] Add native macOS QuickLook/Thumbnailing integration via `objc2` bindings <!-- id: 6.2 -->
- [x] Profile scroll speed and optimize transitions to ensure solid 60 FPS <!-- id: 6.3 -->

## Resolved Issues
- [x] Fix Svelte file type errors by disabling checkJs in jsconfig.json <!-- id: 7.1 -->
- [x] Fix "Add Folder to Scan" dialog not opening by adding dialog capability to default.json <!-- id: 7.2 -->
- [x] Fix thumbnail wrong orientation by extracting EXIF orientation and applying CSS transforms <!-- id: 7.3 -->
- [x] Improve icons across the application by integrating lucide-svelte package <!-- id: 7.4 -->
- [x] Add fullscreen toggle for both the media viewer and the main app window <!-- id: 7.5 -->
- [x] Fix zoom-in icon using Lucide ZoomIn component <!-- id: 7.6 -->
- [x] Fix fullscreen on image/video not working by switching to native Tauri AppWindow fullscreen API <!-- id: 7.7 -->
- [x] Fix wrong initial image viewer orientation by removing the browser double-rotation issue <!-- id: 7.8 -->
- [x] Add manual rotation button (`RotateCw`) for user-driven orientation correction <!-- id: 7.9 -->
- [x] Fix scrolling and thumbnail generation UI hanging by shifting heavy image operations to Tauri's async runtime spawn_blocking <!-- id: 7.10 -->
- [x] Add smooth high-performance CSS transitions, Svelte mount animations, and micro-interactions <!-- id: 7.11 -->
- [x] Fix fullscreen button not working by adding window capabilities (core:window:allow-set-fullscreen and core:window:allow-is-fullscreen) to default.json <!-- id: 7.12 -->
- [x] Fix fullscreen media scaling layout bug (media now expands to fill 100% viewport and controls auto-hide) <!-- id: 7.13 -->
- [x] Fix fullscreen controls overlapping macOS menu bar by switching to a floating card layout offset from the screen top <!-- id: 7.14 -->
- [x] Add high-contrast light-themed logo.svg for dark-mode sidebar visibility and implement Svelte startup splash screen animation <!-- id: 7.15 -->
- [x] Add dynamic search option for media files and floating metadata details drawer with "Reveal in File Manager" action <!-- id: 7.16 -->
- [x] Improve search input icon spacing gap and increase sidebar header logo dimensions for dark mode legibility <!-- id: 7.17 -->
- [x] Add camera metadata, device details, and GPS location coordinates support to file info panel <!-- id: 7.18 -->
- [x] Replace default Tauri system icons with custom Glide light-themed assets across native dock, taskbar, ICNS, and ICO targets <!-- id: 7.19 -->
- [x] Add Cargo release profile size optimizations (LTO, strip, codegen-units, panic = abort) to shrink the final compiled app binary size <!-- id: 7.20 -->
- [x] Add publisher, copyright, author email, and github repo metadata parameters to app packaging templates <!-- id: 7.21 -->
- [x] Fix Svelte selector scope to position the sidebar search icon inside the input field <!-- id: 7.22 -->
