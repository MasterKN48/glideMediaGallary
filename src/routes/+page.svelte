<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fade } from "svelte/transition";
  
  // Svelte Component imports
  import VirtualGrid from "../lib/components/VirtualGrid.svelte";
  import MediaPlayer from "../lib/components/MediaPlayer.svelte";

  // Lucide Icons imports
  import { 
    Images, 
    FolderPlus, 
    Folder, 
    Trash2, 
    Layers, 
    Image as ImageIcon, 
    Video as VideoIcon, 
    Music as MusicIcon, 
    Loader2, 
    CheckCircle2,
    Maximize2,
    Minimize2,
    Search,
    X,
    Sun,
    Moon
  } from "lucide-svelte";

  // App window instance
  const appWindow = getCurrentWindow();

  // App state using Svelte 5 runes
  let folders = $state([]);
  let media = $state([]);
  let activeFilter = $state("all"); // "all" | "image" | "video" | "audio"
  let viewMode = $state("day"); // "day" | "month" | "year"
  let isWindowFullscreen = $state(false);
  
  // Scanning state
  let scanningStatus = $state("");
  let isScanning = $state(false);

  // Selected item for modal viewer
  let selectedItem = $state(null);
  let isInitializing = $state(true);
  let searchQuery = $state("");

  // Theme state — initialize from localStorage to prevent flash
  const initialTheme = typeof localStorage !== "undefined"
    ? (localStorage.getItem("glide-theme") || "dark")
    : "dark";
  let theme = $state(initialTheme);

  // Set attribute immediately before first paint
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", initialTheme);
  }

  // Reactively sync theme changes to DOM + localStorage
  $effect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("glide-theme", theme);
  });

  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
  }

  // Filtered media list based on sidebar filters and search query
  let filteredMedia = $derived(
    media.filter(item => {
      const matchesFilter = activeFilter === "all" || item.media_type === activeFilter;
      const matchesSearch = searchQuery === "" || item.filename.toLowerCase().includes(searchQuery.toLowerCase());
      return matchesFilter && matchesSearch;
    })
  );

  // Navigation indexes for full screen viewer
  let currentIndex = $derived(
    selectedItem ? filteredMedia.findIndex(item => item.file_path === selectedItem.file_path) : -1
  );

  let nextItem = $derived(
    currentIndex !== -1 && currentIndex < filteredMedia.length - 1 ? filteredMedia[currentIndex + 1] : null
  );

  let prevItem = $derived(
    currentIndex > 0 ? filteredMedia[currentIndex - 1] : null
  );

  async function loadInitialData() {
    try {
      const [dbFolders, dbMedia] = await invoke("init_app");
      folders = dbFolders;
      media = dbMedia;
    } catch (err) {
      console.error("Failed to load initial data:", err);
    }
  }

  async function handleAddFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Media Folder to Scan"
      });
      if (selected) {
        const updatedFolders = await invoke("add_folder", { path: selected });
        folders = updatedFolders;
      }
    } catch (err) {
      console.error("Failed to add folder:", err);
    }
  }

  async function handleRemoveFolder(path) {
    try {
      const updatedFolders = await invoke("remove_folder", { path });
      folders = updatedFolders;
      media = media.filter(item => !item.file_path.startsWith(path));
    } catch (err) {
      console.error("Failed to remove folder:", err);
    }
  }

  async function toggleWindowFullscreen() {
    try {
      const isFull = await appWindow.isFullscreen();
      await appWindow.setFullscreen(!isFull);
      isWindowFullscreen = !isFull;
    } catch (err) {
      console.error("Failed to toggle window fullscreen:", err);
      // HTML5 fullscreen fallback
      if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen();
        isWindowFullscreen = true;
      } else {
        document.exitFullscreen();
        isWindowFullscreen = false;
      }
    }
  }

  onMount(() => {
    const startTime = Date.now();

    loadInitialData().then(() => {
      const elapsed = Date.now() - startTime;
      const delay = Math.max(1200 - elapsed, 0);
      setTimeout(() => {
        isInitializing = false;
      }, delay);
    }).catch(() => {
      isInitializing = false;
    });

    // Check initial window fullscreen state
    appWindow.isFullscreen().then(val => {
      isWindowFullscreen = val;
    });

    // Listen to background scanning progress
    const unsubBatch = listen("scanned_batch", (event) => {
      const batch = event.payload;
      // Merge batch into media state. Ensure no duplicate file paths
      media = [...media, ...batch].reduce((acc, current) => {
        const exists = acc.find(item => item.file_path === current.file_path);
        if (!exists) {
          acc.push(current);
        } else {
          // Keep the one with thumbnail if available
          if (current.thumbnail_path) {
            const idx = acc.indexOf(exists);
            acc[idx] = current;
          }
        }
        return acc;
      }, []);
    });

    const unsubStatus = listen("scan_status", (event) => {
      scanningStatus = event.payload;
      if (scanningStatus.includes("started")) {
        isScanning = true;
      } else if (scanningStatus.includes("completed")) {
        isScanning = false;
        setTimeout(() => {
          if (!isScanning) scanningStatus = "";
        }, 5000);
      }
    });

    const unsubError = listen("scan_error", (event) => {
      scanningStatus = `Error: ${event.payload}`;
      isScanning = false;
    });

    return () => {
      unsubBatch.then(f => f());
      unsubStatus.then(f => f());
      unsubError.then(f => f());
    };
  });
</script>

<main class="app-layout">
  <!-- Glassmorphic Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <img src="/logo.svg" class="logo-image" alt="Glide Logo" />
      <div class="header-text">
        <h1>Glide</h1>
        <p class="tagline">Ultra-fast portable media gallery</p>
      </div>
      <button class="theme-toggle-btn" onclick={toggleTheme} title="Toggle theme">
        {#if theme === "dark"}
          <Sun size={18} />
        {:else}
          <Moon size={18} />
        {/if}
      </button>
    </div>

    <!-- Scan Progress Status -->
    {#if scanningStatus}
      <div class="scan-status-card {isScanning ? 'pulse-border' : ''}">
        <div class="status-indicator">
          {#if isScanning}
            <Loader2 class="spin-icon" size={16} />
            <span class="status-label">Scanning Disk...</span>
          {:else}
            <CheckCircle2 class="done-icon" size={16} />
            <span class="status-label">Scan Complete</span>
          {/if}
        </div>
        <div class="status-details">{scanningStatus}</div>
      </div>
    {/if}

    <button class="add-folder-btn" onclick={handleAddFolder}>
      <FolderPlus class="btn-icon" size={18} />
      Add Folder to Scan
    </button>

    <div class="search-box">
      <Search size={16} class="search-icon" />
      <input 
        type="text" 
        placeholder="Search media by name..." 
        bind:value={searchQuery}
        class="search-input"
      />
      {#if searchQuery}
        <button class="clear-search-btn" onclick={() => searchQuery = ""} title="Clear Search">
          <X size={14} />
        </button>
      {/if}
    </div>

    <div class="section-title">Indexed Folders</div>
    <div class="folders-list">
      {#each folders as folder}
        <div class="folder-item" title={folder}>
          <Folder size={14} class="folder-icon" />
          <span class="folder-path">{folder}</span>
          <button class="remove-folder-btn" onclick={() => handleRemoveFolder(folder)} title="Remove folder">
            <Trash2 size={14} />
          </button>
        </div>
      {/each}
      {#if folders.length === 0}
        <div class="no-folders-msg">No folders added. Click above to scan a folder.</div>
      {/if}
    </div>

    <div class="section-title">Filter Library</div>
    <nav class="filter-menu">
      <button class="filter-item {activeFilter === 'all' ? 'active' : ''}" onclick={() => activeFilter = "all"}>
        <Layers class="nav-icon" size={18} />
        All Media
      </button>
      <button class="filter-item {activeFilter === 'image' ? 'active' : ''}" onclick={() => activeFilter = "image"}>
        <ImageIcon class="nav-icon" size={18} />
        Photos
      </button>
      <button class="filter-item {activeFilter === 'video' ? 'active' : ''}" onclick={() => activeFilter = "video"}>
        <VideoIcon class="nav-icon" size={18} />
        Videos
      </button>
      <button class="filter-item {activeFilter === 'audio' ? 'active' : ''}" onclick={() => activeFilter = "audio"}>
        <MusicIcon class="nav-icon" size={18} />
        Audio
      </button>
    </nav>
  </aside>

  <!-- Gallery Content -->
  <section class="gallery-main">
    <header class="gallery-header">
      <div class="header-left">
        <h2>Timeline View</h2>
        <span class="media-total">{filteredMedia.length} files</span>
      </div>
      
      <div class="header-right-controls">
        <div class="view-mode-selector">
          <button class="view-mode-btn {viewMode === 'day' ? 'active' : ''}" onclick={() => viewMode = 'day'}>Day</button>
          <button class="view-mode-btn {viewMode === 'month' ? 'active' : ''}" onclick={() => viewMode = 'month'}>Month</button>
          <button class="view-mode-btn {viewMode === 'year' ? 'active' : ''}" onclick={() => viewMode = 'year'}>Year</button>
        </div>

        <button class="window-fs-btn" onclick={toggleWindowFullscreen} title="Toggle App Fullscreen">
          {#if isWindowFullscreen}
            <Minimize2 size={16} />
          {:else}
            <Maximize2 size={16} />
          {/if}
        </button>
      </div>
    </header>

    <div class="grid-content">
      <VirtualGrid 
        mediaItems={filteredMedia} 
        {viewMode} 
        onSelectItem={(item) => selectedItem = item} 
      />
    </div>
  </section>

  <!-- Full Screen Viewer Modal -->
  {#if selectedItem}
    <MediaPlayer 
      item={selectedItem} 
      onNext={nextItem ? () => selectedItem = nextItem : null} 
      onPrev={prevItem ? () => selectedItem = prevItem : null} 
      onClose={() => selectedItem = null} 
    />
  {/if}
</main>

{#if isInitializing}
  <div class="splash-screen" transition:fade={{ duration: 300 }}>
    <div class="splash-content">
      <img src="/logo.svg" class="splash-logo" alt="Glide Logo" />
      <h1 class="splash-title">Glide</h1>
      <p class="splash-tagline">Initializing media library...</p>
      <div class="splash-spinner">
        <div class="spinner-bar"></div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ===== Design Token System ===== */
  :global(:root) {
    /* Backgrounds */
    --bg-primary: #060608;
    --bg-gradient-start: #07070a;
    --bg-gradient-end: #0d0d12;
    --bg-splash: #07070a;

    /* Sidebar */
    --sidebar-bg: rgba(13, 13, 18, 0.45);
    --sidebar-border: rgba(255, 255, 255, 0.05);

    /* Text hierarchy */
    --text-primary: #ffffff;
    --text-secondary: #e2e2e9;
    --text-muted: #a8a8af;
    --text-faint: #88888e;
    --text-subtle: #c5c6c7;
    --text-placeholder: #66666e;

    /* Accent (teal/cyan) */
    --accent: #66fcf1;
    --accent-bg: rgba(102, 252, 241, 0.08);
    --accent-bg-hover: rgba(102, 252, 241, 0.15);
    --accent-border: rgba(102, 252, 241, 0.2);
    --accent-border-hover: rgba(102, 252, 241, 0.4);
    --accent-glow: rgba(102, 252, 241, 0.15);
    --accent-glow-strong: rgba(102, 252, 241, 0.45);
    --accent-subtle: rgba(102, 252, 241, 0.1);
    --accent-dim: rgba(102, 252, 241, 0.04);
    --accent-fill: rgba(102, 252, 241, 0.3);

    /* Surfaces */
    --surface: rgba(255, 255, 255, 0.02);
    --surface-border: rgba(255, 255, 255, 0.04);
    --surface-hover: rgba(255, 255, 255, 0.04);
    --surface-elevated: rgba(255, 255, 255, 0.03);
    --surface-border-elevated: rgba(255, 255, 255, 0.05);
    --surface-hover-strong: rgba(255, 255, 255, 0.05);
    --surface-strong: rgba(255, 255, 255, 0.08);

    /* Overlays */
    --overlay-bg: rgba(3, 3, 5, 0.95);
    --overlay-bar: rgba(0, 0, 0, 0.2);
    --overlay-panel: rgba(13, 13, 18, 0.85);
    --overlay-float: rgba(13, 13, 18, 0.8);

    /* Gallery header */
    --header-bg: rgba(7, 7, 10, 0.2);

    /* Shadows */
    --shadow-strong: rgba(0, 0, 0, 0.8);
    --shadow-medium: rgba(0, 0, 0, 0.5);
    --shadow-light: rgba(0, 0, 0, 0.25);
    --shadow-text: 0 2px 4px rgba(0, 0, 0, 0.5);

    /* Media placeholders */
    --placeholder-bg: rgba(20, 20, 25, 0.7);
    --audio-card-bg: rgba(31, 40, 51, 0.45);

    /* Skeleton loader */
    --skeleton-start: rgba(255, 255, 255, 0.01);
    --skeleton-mid: rgba(255, 255, 255, 0.05);

    /* Scrollbar */
    --scrollbar-track: rgba(0, 0, 0, 0.1);
    --scrollbar-thumb: rgba(102, 252, 241, 0.2);
    --scrollbar-thumb-hover: rgba(102, 252, 241, 0.4);

    /* Status colors */
    --danger: #ff5f56;
    --danger-bg: rgba(255, 95, 86, 0.08);
    --danger-border: rgba(255, 95, 86, 0.3);
    --success: #39d353;
    --success-glow: rgba(57, 211, 83, 0.3);

    /* Logo effects */
    --logo-filter: drop-shadow(0 0 8px rgba(102, 252, 241, 0.45));
    --logo-pulse-start: drop-shadow(0 0 8px rgba(102, 252, 241, 0.4));
    --logo-pulse-end: drop-shadow(0 0 24px rgba(102, 252, 241, 0.8));
  }

  /* ===== Light Theme Overrides ===== */
  :global([data-theme="light"]) {
    --bg-primary: #f5f5f7;
    --bg-gradient-start: #eeeef0;
    --bg-gradient-end: #f9f9fb;
    --bg-splash: #f5f5f7;

    --sidebar-bg: rgba(255, 255, 255, 0.7);
    --sidebar-border: rgba(0, 0, 0, 0.08);

    --text-primary: #1a1a1e;
    --text-secondary: #3a3a42;
    --text-muted: #6e6e78;
    --text-faint: #8e8e96;
    --text-subtle: #5a5a64;
    --text-placeholder: #9e9ea6;

    --accent: #0d9488;
    --accent-bg: rgba(13, 148, 136, 0.08);
    --accent-bg-hover: rgba(13, 148, 136, 0.15);
    --accent-border: rgba(13, 148, 136, 0.25);
    --accent-border-hover: rgba(13, 148, 136, 0.4);
    --accent-glow: rgba(13, 148, 136, 0.1);
    --accent-glow-strong: rgba(13, 148, 136, 0.3);
    --accent-subtle: rgba(13, 148, 136, 0.08);
    --accent-dim: rgba(13, 148, 136, 0.04);
    --accent-fill: rgba(13, 148, 136, 0.2);

    --surface: rgba(0, 0, 0, 0.03);
    --surface-border: rgba(0, 0, 0, 0.06);
    --surface-hover: rgba(0, 0, 0, 0.06);
    --surface-elevated: rgba(0, 0, 0, 0.04);
    --surface-border-elevated: rgba(0, 0, 0, 0.08);
    --surface-hover-strong: rgba(0, 0, 0, 0.08);
    --surface-strong: rgba(0, 0, 0, 0.06);

    --overlay-bg: rgba(255, 255, 255, 0.95);
    --overlay-bar: rgba(0, 0, 0, 0.04);
    --overlay-panel: rgba(255, 255, 255, 0.92);
    --overlay-float: rgba(255, 255, 255, 0.88);

    --header-bg: rgba(255, 255, 255, 0.5);

    --shadow-strong: rgba(0, 0, 0, 0.15);
    --shadow-medium: rgba(0, 0, 0, 0.1);
    --shadow-light: rgba(0, 0, 0, 0.08);
    --shadow-text: none;

    --placeholder-bg: rgba(240, 240, 242, 0.9);
    --audio-card-bg: rgba(255, 255, 255, 0.8);

    --skeleton-start: rgba(0, 0, 0, 0.03);
    --skeleton-mid: rgba(0, 0, 0, 0.06);

    --scrollbar-track: rgba(0, 0, 0, 0.05);
    --scrollbar-thumb: rgba(13, 148, 136, 0.2);
    --scrollbar-thumb-hover: rgba(13, 148, 136, 0.4);

    --logo-filter: drop-shadow(0 0 6px rgba(13, 148, 136, 0.3));
    --logo-pulse-start: drop-shadow(0 0 6px rgba(13, 148, 136, 0.3));
    --logo-pulse-end: drop-shadow(0 0 16px rgba(13, 148, 136, 0.5));
  }

  /* ===== Global Resets ===== */
  :global(body) {
      margin: 0;
      padding: 0;
      background-color: var(--bg-primary);
      color: var(--text-secondary);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
      overflow: hidden;
      height: 100vh;
  }
  
  :global(*) {
      box-sizing: border-box;
  }

  .app-layout {
      display: flex;
      width: 100vw;
      height: 100vh;
      overflow: hidden;
      background: linear-gradient(135deg, var(--bg-gradient-start) 0%, var(--bg-gradient-end) 100%);
  }
  
  /* Glassmorphism Sidebar */
  .sidebar {
      width: 280px;
      height: 100%;
      background: var(--sidebar-bg);
      backdrop-filter: blur(25px);
      -webkit-backdrop-filter: blur(25px);
      border-right: 1px solid var(--sidebar-border);
      padding: 24px;
      display: flex;
      flex-direction: column;
      flex-shrink: 0;
      z-index: 10;
  }
  
  .sidebar-header {
      display: flex;
      align-items: center;
      gap: 14px;
      margin-bottom: 30px;
  }
  
  .header-text {
      display: flex;
      flex-direction: column;
      gap: 2px;
      flex: 1;
  }
  
  .tagline {
      font-size: 0.68rem;
      color: var(--text-muted);
      margin: 0;
      line-height: 1.35;
      font-weight: 500;
  }

  .logo-image {
      width: 46px;
      height: 46px;
      flex-shrink: 0;
      filter: var(--logo-filter);
  }
  
  .sidebar-header h1 {
      font-size: 1.35rem;
      font-weight: 700;
      color: var(--text-primary);
      margin: 0;
      letter-spacing: -0.5px;
      line-height: 1.1;
  }

  /* Theme Toggle Button */
  .theme-toggle-btn {
      background: var(--surface-elevated);
      border: 1px solid var(--surface-border-elevated);
      border-radius: 8px;
      width: 34px;
      height: 34px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text-muted);
      cursor: pointer;
      transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
      outline: none;
      flex-shrink: 0;
  }
  .theme-toggle-btn:hover {
      color: var(--accent);
      border-color: var(--accent-border-hover);
      background: var(--accent-bg);
      transform: rotate(15deg);
  }
  
  .scan-status-card {
      background: var(--accent-dim);
      border: 1px solid var(--accent-subtle);
      border-radius: 12px;
      padding: 12px;
      margin-bottom: 20px;
      font-size: 0.8rem;
      transition: all 0.3s;
  }
  
  .pulse-border {
      animation: borderGlow 2s infinite alternate;
  }
  
  @keyframes borderGlow {
      0% { border-color: var(--accent-subtle); }
      100% { border-color: var(--accent-glow-strong); }
  }
  
  .status-indicator {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 6px;
  }
  
  :global(.spin-icon) {
      color: var(--accent);
      animation: spin 1.5s linear infinite;
  }
  
  @keyframes spin {
      100% { transform: rotate(360deg); }
  }

  :global(.done-icon) {
      color: var(--success);
      filter: drop-shadow(0 0 3px var(--success-glow));
  }
  
  .status-label {
      font-weight: 600;
      color: var(--text-primary);
  }
  
  .status-details {
      color: var(--text-muted);
      word-break: break-all;
      line-height: 1.3;
  }
  
  .add-folder-btn {
      width: 100%;
      background: var(--accent-bg);
      color: var(--accent);
      border: 1px solid var(--accent-border);
      border-radius: 10px;
      padding: 10px 16px;
      font-size: 0.9rem;
      font-weight: 600;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      cursor: pointer;
      transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
      margin-bottom: 24px;
      outline: none;
  }
  .add-folder-btn:hover {
      background: var(--accent-bg-hover);
      border-color: var(--accent-border-hover);
      box-shadow: 0 0 12px var(--accent-glow);
      transform: translateY(-1px);
  }
  .add-folder-btn:active {
      transform: translateY(0);
  }
  
  .section-title {
      font-size: 0.75rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 1px;
      color: var(--text-muted);
      margin-bottom: 12px;
  }
  
  .folders-list {
      display: flex;
      flex-direction: column;
      gap: 8px;
      margin-bottom: 28px;
      max-height: 180px;
      overflow-y: auto;
  }
  
  .folder-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: var(--surface);
      border: 1px solid var(--surface-border);
      border-radius: 8px;
      padding: 8px 12px;
      gap: 8px;
      transition: background 0.25s ease, transform 0.25s cubic-bezier(0.25, 1, 0.5, 1);
  }
  .folder-item:hover {
      background: var(--surface-hover);
      transform: translateX(4px);
  }

  .folder-icon {
      color: var(--text-muted);
      flex-shrink: 0;
  }
  
  .folder-path {
      font-size: 0.8rem;
      color: var(--text-subtle);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      flex: 1;
  }
  
  .remove-folder-btn {
      background: none;
      border: none;
      padding: 0;
      color: var(--text-faint);
      cursor: pointer;
      width: 18px;
      height: 18px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: color 0.2s ease, transform 0.2s cubic-bezier(0.25, 1, 0.5, 1);
  }
  .remove-folder-btn:hover {
      color: var(--danger);
      transform: scale(1.15) rotate(5deg);
  }
  
  .no-folders-msg {
      font-size: 0.78rem;
      color: var(--text-faint);
      line-height: 1.4;
      text-align: center;
      padding: 10px 0;
  }
  
  .filter-menu {
      display: flex;
      flex-direction: column;
      gap: 4px;
  }
  
  .filter-item {
      background: none;
      border: none;
      width: 100%;
      text-align: left;
      padding: 10px 12px;
      border-radius: 8px;
      color: var(--text-subtle);
      font-size: 0.9rem;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 12px;
      cursor: pointer;
      transition: background 0.25s ease, color 0.25s ease, padding-left 0.25s cubic-bezier(0.25, 1, 0.5, 1);
      outline: none;
  }
  .filter-item:hover {
      background: var(--surface-elevated);
      color: var(--text-primary);
      padding-left: 18px;
  }
  .filter-item.active {
      background: var(--accent-bg);
      color: var(--accent);
      border-left: 3px solid var(--accent);
      border-radius: 0 8px 8px 0;
  }
  .nav-icon {
      flex-shrink: 0;
  }
  
  /* Gallery Main Panel */
  .gallery-main {
      flex: 1;
      height: 100%;
      display: flex;
      flex-direction: column;
      overflow: hidden;
  }
  
  .gallery-header {
      height: 70px;
      border-bottom: 1px solid var(--surface-border-elevated);
      padding: 0 32px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: var(--header-bg);
  }
  
  .header-left h2 {
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--text-primary);
      margin: 0 0 2px 0;
      letter-spacing: -0.3px;
  }
  .media-total {
      font-size: 0.75rem;
      color: var(--text-muted);
  }
  
  .header-right-controls {
      display: flex;
      align-items: center;
      gap: 12px;
  }

  .view-mode-selector {
      display: flex;
      background: var(--surface-elevated);
      border: 1px solid var(--surface-border-elevated);
      border-radius: 8px;
      padding: 3px;
  }
  
  .view-mode-btn {
      background: none;
      border: none;
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s;
      outline: none;
  }
  .view-mode-btn:hover {
      color: var(--text-primary);
  }
  .view-mode-btn.active {
      background: var(--accent-subtle);
      color: var(--accent);
      box-shadow: 0 2px 8px var(--shadow-light);
  }
  
  .window-fs-btn {
      background: var(--surface-elevated);
      border: 1px solid var(--surface-border-elevated);
      border-radius: 8px;
      width: 34px;
      height: 34px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text-muted);
      cursor: pointer;
      transition: all 0.2s;
      outline: none;
  }
  .window-fs-btn:hover {
      color: var(--accent);
      border-color: var(--accent-border-hover);
      background: var(--accent-bg);
  }

  .grid-content {
      flex: 1;
      overflow: hidden;
  }

  /* Opening Splash Screen */
  .splash-screen {
      position: fixed;
      top: 0;
      left: 0;
      width: 100vw;
      height: 100vh;
      background: var(--bg-splash);
      z-index: 9999;
      display: flex;
      align-items: center;
      justify-content: center;
      user-select: none;
  }
  
  .splash-content {
      display: flex;
      flex-direction: column;
      align-items: center;
      text-align: center;
  }
  
  .splash-logo {
      width: 96px;
      height: 96px;
      margin-bottom: 24px;
      animation: pulseGlow 2.5s infinite alternate ease-in-out;
  }
  
  @keyframes pulseGlow {
      0% { transform: scale(1); filter: var(--logo-pulse-start); }
      100% { transform: scale(1.06); filter: var(--logo-pulse-end); }
  }
  
  .splash-title {
      font-size: 2.2rem;
      font-weight: 800;
      color: var(--text-primary);
      margin: 0 0 8px 0;
      letter-spacing: -1px;
      font-family: system-ui, sans-serif;
  }
  
  .splash-tagline {
      font-size: 0.9rem;
      color: var(--text-muted);
      margin: 0 0 32px 0;
  }
  
  .splash-spinner {
      width: 160px;
      height: 3px;
      background: var(--surface-border-elevated);
      border-radius: 2px;
      overflow: hidden;
      position: relative;
  }
  
  .spinner-bar {
      width: 60px;
      height: 100%;
      background: var(--accent);
      position: absolute;
      border-radius: 2px;
      animation: loadingBar 1.5s infinite ease-in-out;
      box-shadow: 0 0 8px var(--accent);
  }
  
  @keyframes loadingBar {
      0% { left: -60px; }
      100% { left: 160px; }
  }

  /* Glassmorphic Search Box */
  .search-box {
      position: relative;
      width: 100%;
      margin-bottom: 24px;
      display: flex;
      align-items: center;
  }
  
  .search-icon {
      position: absolute;
      left: 14px;
      color: var(--text-faint);
      pointer-events: none;
      transition: color 0.25s;
  }
  
  .search-input {
      width: 100%;
      background: var(--surface);
      border: 1px solid var(--surface-border);
      border-radius: 10px;
      padding: 10px 36px 10px 42px;
      color: var(--text-primary);
      font-size: 0.82rem;
      outline: none;
      transition: all 0.25s cubic-bezier(0.25, 1, 0.5, 1);
  }
  
  .search-input::placeholder {
      color: var(--text-placeholder);
  }
  
  .search-input:focus {
      background: var(--surface-hover-strong);
      border-color: var(--accent-border-hover);
      box-shadow: 0 0 12px var(--accent-glow);
  }
  
  .search-input:focus + :global(.search-icon) {
      color: var(--accent);
  }
  
  .clear-search-btn {
      position: absolute;
      right: 10px;
      background: none;
      border: none;
      color: var(--text-faint);
      cursor: pointer;
      padding: 0;
      width: 18px;
      height: 18px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
      transition: all 0.2s;
  }
  
  .clear-search-btn:hover {
      color: var(--text-primary);
      background: var(--surface-strong);
  }
</style>
