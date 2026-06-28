<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  
  import VirtualGrid from "../lib/components/VirtualGrid.svelte";
  import MediaPlayer from "../lib/components/MediaPlayer.svelte";

  // App state using Svelte 5 runes
  let folders = $state([]);
  let media = $state([]);
  let activeFilter = $state("all"); // "all" | "image" | "video" | "audio"
  let viewMode = $state("day"); // "day" | "month" | "year"
  
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

  onMount(() => {
    loadInitialData();

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
      <svg viewBox="0 0 24 24" class="logo-icon"><path d="M22 16V4c0-1.1-.9-2-2-2H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2zm-11-4l2.03 2.71L16 11l4 5H8l3-4zM2 6v14c0 1.1.9 2 2 2h14v-2H4V6H2z"/></svg>
      <h1>Media Gallery</h1>
    </div>

    <!-- Scan Progress Status -->
    {#if scanningStatus}
      <div class="scan-status-card {isScanning ? 'pulse-border' : ''}">
        <div class="status-indicator">
          <span class="status-dot {isScanning ? 'active' : ''}"></span>
          <span class="status-label">{isScanning ? 'Scanning Disk...' : 'Done'}</span>
        </div>
        <div class="status-details">{scanningStatus}</div>
      </div>
    {/if}

    <button class="add-folder-btn" onclick={handleAddFolder}>
      <svg viewBox="0 0 24 24" class="btn-icon"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
      Add Folder to Scan
    </button>

    <div class="section-title">Indexed Folders</div>
    <div class="folders-list">
      {#each folders as folder}
        <div class="folder-item" title={folder}>
          <span class="folder-path">{folder}</span>
          <button class="remove-folder-btn" onclick={() => handleRemoveFolder(folder)} title="Remove folder">
            <svg viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
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
        <svg class="nav-icon" viewBox="0 0 24 24"><path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H8V4h12v12z"/></svg>
        All Media
      </button>
      <button class="filter-item {activeFilter === 'image' ? 'active' : ''}" onclick={() => activeFilter = "image"}>
        <svg class="nav-icon" viewBox="0 0 24 24"><path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/></svg>
        Photos
      </button>
      <button class="filter-item {activeFilter === 'video' ? 'active' : ''}" onclick={() => activeFilter = "video"}>
        <svg class="nav-icon" viewBox="0 0 24 24"><path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/></svg>
        Videos
      </button>
      <button class="filter-item {activeFilter === 'audio' ? 'active' : ''}" onclick={() => activeFilter = "audio"}>
        <svg class="nav-icon" viewBox="0 0 24 24"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
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
      
      <div class="view-mode-selector">
        <button class="view-mode-btn {viewMode === 'day' ? 'active' : ''}" onclick={() => viewMode = 'day'}>Day</button>
        <button class="view-mode-btn {viewMode === 'month' ? 'active' : ''}" onclick={() => viewMode = 'month'}>Month</button>
        <button class="view-mode-btn {viewMode === 'year' ? 'active' : ''}" onclick={() => viewMode = 'year'}>Year</button>
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
      align-items: center;
      gap: 12px;
      margin-bottom: 30px;
  }
  
  .logo-icon {
      width: 24px;
      height: 24px;
      fill: #66fcf1;
      filter: drop-shadow(0 0 6px rgba(102, 252, 241, 0.5));
  }
  
  .sidebar-header h1 {
      font-size: 1.25rem;
      font-weight: 700;
      color: #ffffff;
      margin: 0;
      letter-spacing: -0.5px;
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
  
  .status-dot {
      width: 8px;
      height: 8px;
      background: #88888e;
      border-radius: 50%;
  }
  .status-dot.active {
      background: #66fcf1;
      box-shadow: 0 0 8px #66fcf1;
      animation: pulseDot 1.5s infinite;
  }
  
  @keyframes pulseDot {
      0% { transform: scale(0.9); opacity: 0.6; }
      50% { transform: scale(1.15); opacity: 1; }
      100% { transform: scale(0.9); opacity: 0.6; }
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
  .btn-icon {
      width: 16px;
      height: 16px;
      fill: currentColor;
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
      transition: background 0.2s;
  }
  .folder-item:hover {
      background: rgba(255, 255, 255, 0.04);
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
      transition: color 0.2s;
  }
  .remove-folder-btn:hover {
      color: #ff5f56;
  }
  .remove-folder-btn svg {
      width: 14px;
      height: 14px;
      fill: currentColor;
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
      transition: all 0.2s;
      outline: none;
  }
  .filter-item:hover {
      background: rgba(255, 255, 255, 0.03);
      color: #ffffff;
  }
  .filter-item.active {
      background: rgba(102, 252, 241, 0.08);
      color: #66fcf1;
      border-left: 3px solid #66fcf1;
      border-radius: 0 8px 8px 0;
  }
  .nav-icon {
      width: 18px;
      height: 18px;
      fill: currentColor;
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
  
  .grid-content {
      flex: 1;
      overflow: hidden;
  }
</style>
