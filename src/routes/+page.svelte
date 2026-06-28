<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  
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
    Minimize2
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

  // Filtered media list based on sidebar filters
  let filteredMedia = $derived(
    activeFilter === "all" 
      ? media 
      : media.filter(item => item.media_type === activeFilter)
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
    loadInitialData();

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

<style>
  :global(body) {
      margin: 0;
      padding: 0;
      background-color: #060608;
      color: #e2e2e9;
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
      background: linear-gradient(135deg, #07070a 0%, #0d0d12 100%);
  }
  
  /* Glassmorphism Sidebar */
  .sidebar {
      width: 280px;
      height: 100%;
      background: rgba(13, 13, 18, 0.45);
      backdrop-filter: blur(25px);
      -webkit-backdrop-filter: blur(25px);
      border-right: 1px solid rgba(255, 255, 255, 0.05);
      padding: 24px;
      display: flex;
      flex-direction: column;
      flex-shrink: 0;
      z-index: 10;
  }
  
  .sidebar-header {
      display: flex;
      align-items: flex-start;
      gap: 12px;
      margin-bottom: 30px;
  }
  
  .header-text {
      display: flex;
      flex-direction: column;
      gap: 2px;
  }
  
  .tagline {
      font-size: 0.68rem;
      color: #a8a8af;
      margin: 0;
      line-height: 1.35;
      font-weight: 500;
  }

  .logo-image {
      width: 26px;
      height: 26px;
      flex-shrink: 0;
      filter: drop-shadow(0 0 4px rgba(102, 252, 241, 0.4));
      margin-top: 2px;
  }
  
  .sidebar-header h1 {
      font-size: 1.25rem;
      font-weight: 700;
      color: #ffffff;
      margin: 0;
      letter-spacing: -0.5px;
      line-height: 1.1;
  }
  
  .scan-status-card {
      background: rgba(102, 252, 241, 0.04);
      border: 1px solid rgba(102, 252, 241, 0.1);
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
      0% { border-color: rgba(102, 252, 241, 0.1); }
      100% { border-color: rgba(102, 252, 241, 0.45); }
  }
  
  .status-indicator {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 6px;
  }
  
  :global(.spin-icon) {
      color: #66fcf1;
      animation: spin 1.5s linear infinite;
  }
  
  @keyframes spin {
      100% { transform: rotate(360deg); }
  }

  :global(.done-icon) {
      color: #39d353;
      filter: drop-shadow(0 0 3px rgba(57, 211, 83, 0.3));
  }
  
  .status-label {
      font-weight: 600;
      color: #ffffff;
  }
  
  .status-details {
      color: #a8a8af;
      word-break: break-all;
      line-height: 1.3;
  }
  
  .add-folder-btn {
      width: 100%;
      background: rgba(102, 252, 241, 0.08);
      color: #66fcf1;
      border: 1px solid rgba(102, 252, 241, 0.2);
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
      background: rgba(102, 252, 241, 0.15);
      border-color: rgba(102, 252, 241, 0.4);
      box-shadow: 0 0 12px rgba(102, 252, 241, 0.15);
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
      color: #a8a8af;
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
      background: rgba(255, 255, 255, 0.02);
      border: 1px solid rgba(255, 255, 255, 0.04);
      border-radius: 8px;
      padding: 8px 12px;
      gap: 8px;
      transition: background 0.25s ease, transform 0.25s cubic-bezier(0.25, 1, 0.5, 1);
  }
  .folder-item:hover {
      background: rgba(255, 255, 255, 0.04);
      transform: translateX(4px);
  }

  .folder-icon {
      color: #a8a8af;
      flex-shrink: 0;
  }
  
  .folder-path {
      font-size: 0.8rem;
      color: #c5c6c7;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      flex: 1;
  }
  
  .remove-folder-btn {
      background: none;
      border: none;
      padding: 0;
      color: #88888e;
      cursor: pointer;
      width: 18px;
      height: 18px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: color 0.2s ease, transform 0.2s cubic-bezier(0.25, 1, 0.5, 1);
  }
  .remove-folder-btn:hover {
      color: #ff5f56;
      transform: scale(1.15) rotate(5deg);
  }
  
  .no-folders-msg {
      font-size: 0.78rem;
      color: #88888e;
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
      color: #c5c6c7;
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
      background: rgba(255, 255, 255, 0.03);
      color: #ffffff;
      padding-left: 18px;
  }
  .filter-item.active {
      background: rgba(102, 252, 241, 0.08);
      color: #66fcf1;
      border-left: 3px solid #66fcf1;
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
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
      padding: 0 32px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: rgba(7, 7, 10, 0.2);
  }
  
  .header-left h2 {
      font-size: 1.25rem;
      font-weight: 700;
      color: #ffffff;
      margin: 0 0 2px 0;
      letter-spacing: -0.3px;
  }
  .media-total {
      font-size: 0.75rem;
      color: #a8a8af;
  }
  
  .header-right-controls {
      display: flex;
      align-items: center;
      gap: 12px;
  }

  .view-mode-selector {
      display: flex;
      background: rgba(255, 255, 255, 0.03);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 8px;
      padding: 3px;
  }
  
  .view-mode-btn {
      background: none;
      border: none;
      color: #a8a8af;
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s;
      outline: none;
  }
  .view-mode-btn:hover {
      color: #ffffff;
  }
  .view-mode-btn.active {
      background: rgba(102, 252, 241, 0.1);
      color: #66fcf1;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  }
  
  .window-fs-btn {
      background: rgba(255, 255, 255, 0.03);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 8px;
      width: 34px;
      height: 34px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #a8a8af;
      cursor: pointer;
      transition: all 0.2s;
      outline: none;
  }
  .window-fs-btn:hover {
      color: #66fcf1;
      border-color: rgba(102, 252, 241, 0.4);
      background: rgba(102, 252, 241, 0.08);
  }

  .grid-content {
      flex: 1;
      overflow: hidden;
  }
</style>
