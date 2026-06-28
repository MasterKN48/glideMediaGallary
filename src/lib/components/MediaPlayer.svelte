<script>
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  
  // Lucide Icons
  import { 
    ZoomIn, 
    ZoomOut, 
    RotateCw, 
    Maximize2, 
    Minimize2, 
    X, 
    ChevronLeft, 
    ChevronRight,
    RefreshCw,
    Music
  } from "lucide-svelte";

  let { item, onNext, onPrev, onClose } = $props();

  let mediaSrc = $derived(convertFileSrc(item.file_path));

  // Image zoom state
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let isDragging = $state(false);
  let startX = 0;
  let startY = 0;

  // Video element binding
  let videoEl = $state(null);
  let isPlaying = $state(false);
  
  // Fullscreen state
  let isFullscreenState = $state(false);
  const appWindow = getCurrentWindow();

  // Manual rotation state
  let manualRotation = $state(0);

  // Reset zoom & manual rotation when switching items
  $effect(() => {
    if (item) {
      manualRotation = 0;
      resetZoom();
    }
  });

  function handleKeyDown(e) {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowRight" && onNext) {
      resetZoom();
      onNext();
    } else if (e.key === "ArrowLeft" && onPrev) {
      resetZoom();
      onPrev();
    } else if (e.key === " ") {
      // Toggle play/pause for video
      if (item.media_type === "video" && videoEl) {
        e.preventDefault();
        if (isPlaying) {
          videoEl.pause();
        } else {
          videoEl.play();
        }
      }
    } else if (e.key === "f" || e.key === "F") {
      toggleTauriFullscreen();
    } else if (e.key === "r" || e.key === "R") {
      rotateCw();
    }
  }

  function resetZoom() {
    zoom = 1;
    panX = 0;
    panY = 0;
  }

  function handleZoomIn() {
    zoom = Math.min(zoom + 0.25, 4);
  }

  function handleZoomOut() {
    zoom = Math.max(zoom - 0.25, 0.5);
  }

  function rotateCw() {
    manualRotation = (manualRotation + 90) % 360;
  }

  async function toggleTauriFullscreen() {
    try {
      const isFull = await appWindow.isFullscreen();
      await appWindow.setFullscreen(!isFull);
      isFullscreenState = !isFull;
    } catch (err) {
      console.error("Failed to toggle Tauri window fullscreen, falling back:", err);
      // HTML5 fullscreen fallback
      if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen().then(() => {
          isFullscreenState = true;
        }).catch((e) => console.error(e));
      } else {
        document.exitFullscreen().then(() => {
          isFullscreenState = false;
        });
      }
    }
  }

  function handleMouseDown(e) {
    if (zoom > 1) {
      isDragging = true;
      startX = e.clientX - panX;
      startY = e.clientY - panY;
    }
  }

  function handleMouseMove(e) {
    if (isDragging) {
      panX = e.clientX - startX;
      panY = e.clientY - startY;
    }
  }

  function handleMouseUp() {
    isDragging = false;
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    
    // Check initial native fullscreen state
    appWindow.isFullscreen().then(val => {
      isFullscreenState = val;
    });

    const onFsChange = () => {
      isFullscreenState = !!document.fullscreenElement;
    };
    document.addEventListener("fullscreenchange", onFsChange);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("fullscreenchange", onFsChange);
    };
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div 
  class="media-player-overlay" 
  role="dialog"
  onmousedown={(e) => { if (e.target === e.currentTarget) onClose(); }}
>
  <!-- Top Bar Controls -->
  <div class="player-top-bar">
    <div class="media-title">
      <span class="file-name">{item.filename}</span>
      <span class="file-size">{(item.size / (1024 * 1024)).toFixed(2)} MB</span>
    </div>
    
    <div class="top-controls">
      {#if item.media_type === "image"}
        <button class="icon-btn" onclick={handleZoomOut} title="Zoom Out">
          <ZoomOut size={18} />
        </button>
        <button class="icon-btn" onclick={handleZoomIn} title="Zoom In">
          <ZoomIn size={18} />
        </button>
        <button class="icon-btn" onclick={resetZoom} title="Reset Zoom">
          <RefreshCw size={18} />
        </button>
      {/if}
      
      {#if item.media_type === "image" || item.media_type === "video"}
        <button class="icon-btn" onclick={rotateCw} title="Rotate 90° CW (R)">
          <RotateCw size={18} />
        </button>
      {/if}

      <button class="icon-btn" onclick={toggleTauriFullscreen} title="Toggle Fullscreen (F)">
        {#if isFullscreenState}
          <Minimize2 size={18} />
        {:else}
          <Maximize2 size={18} />
        {/if}
      </button>

      <button class="close-btn" onclick={onClose} title="Close (Esc)">
        <X size={18} />
      </button>
    </div>
  </div>

  <!-- Navigation Arrows -->
  {#if onPrev}
    <button class="nav-arrow left" onclick={() => { resetZoom(); onPrev(); }} title="Previous">
      <ChevronLeft size={36} />
    </button>
  {/if}
  
  {#if onNext}
    <button class="nav-arrow right" onclick={() => { resetZoom(); onNext(); }} title="Next">
      <ChevronRight size={36} />
    </button>
  {/if}

  <!-- Active Media Container -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="media-content-container"
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
  >
    {#if item.media_type === "image"}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <img 
        src={mediaSrc} 
        alt={item.filename}
        class="viewer-image {manualRotation === 90 || manualRotation === 270 ? 'rotated-90-deg' : ''}"
        style="transform: scale({zoom}) translate({panX}px, {panY}px) rotate({manualRotation}deg); cursor: {zoom > 1 ? 'grab' : 'default'}"
        onmousedown={handleMouseDown}
        draggable="false"
      />
    {:else if item.media_type === "video"}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video 
        bind:this={videoEl}
        src={mediaSrc}
        class="viewer-video"
        style="transform: rotate({manualRotation}deg);"
        controls
        autoplay
        onplay={() => isPlaying = true}
        onpause={() => isPlaying = false}
      ></video>
    {:else if item.media_type === "audio"}
      <div class="viewer-audio-card">
        <Music class="audio-card-icon" size={48} />
        <h3>{item.filename}</h3>
        <p class="audio-meta">{(item.size / (1024 * 1024)).toFixed(2)} MB</p>
        <audio 
          src={mediaSrc}
          class="viewer-audio"
          controls
          autoplay
        ></audio>
      </div>
    {/if}
  </div>
</div>

<style>
  .media-player-overlay {
      position: fixed;
      top: 0;
      left: 0;
      width: 100vw;
      height: 100vh;
      background: rgba(3, 3, 5, 0.95);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
      z-index: 1000;
      display: flex;
      flex-direction: column;
      user-select: none;
  }
  
  .player-top-bar {
      height: 64px;
      padding: 0 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
      background: rgba(0, 0, 0, 0.2);
  }
  
  .media-title {
      display: flex;
      flex-direction: column;
      gap: 2px;
  }
  
  .file-name {
      font-size: 0.95rem;
      font-weight: 600;
      color: #ffffff;
  }
  
  .file-size {
      font-size: 0.75rem;
      color: #a8a8af;
  }
  
  .top-controls {
      display: flex;
      align-items: center;
      gap: 12px;
  }
  
  .icon-btn, .close-btn {
      background: rgba(255, 255, 255, 0.03);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 8px;
      width: 36px;
      height: 36px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #a8a8af;
      cursor: pointer;
      transition: all 0.2s;
      outline: none;
  }
  
  .icon-btn:hover {
      color: #66fcf1;
      border-color: rgba(102, 252, 241, 0.3);
      background: rgba(102, 252, 241, 0.08);
  }
  
  .close-btn:hover {
      color: #ff5f56;
      border-color: rgba(255, 95, 86, 0.3);
      background: rgba(255, 95, 86, 0.08);
  }
  
  .nav-arrow {
      position: absolute;
      top: 50%;
      transform: translateY(-50%);
      background: rgba(255, 255, 255, 0.02);
      border: 1px solid rgba(255, 255, 255, 0.04);
      border-radius: 50%;
      width: 60px;
      height: 60px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #ffffff;
      cursor: pointer;
      z-index: 1010;
      transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
      outline: none;
  }
  .nav-arrow:hover {
      background: rgba(102, 252, 241, 0.1);
      border-color: rgba(102, 252, 241, 0.4);
      color: #66fcf1;
      box-shadow: 0 0 16px rgba(102, 252, 241, 0.2);
  }
  
  .nav-arrow.left {
      left: 32px;
  }
  
  .nav-arrow.right {
      right: 32px;
  }
  
  .media-content-container {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      overflow: hidden;
      padding-top: 60px;
  }
  
  .viewer-image {
      max-width: 85%;
      max-height: 80%;
      object-fit: contain;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8);
      border-radius: 4px;
      transition: transform 0.1s ease-out;
  }
  
  .viewer-image.rotated-90-deg {
      max-width: 55vh;
      max-height: 80vw;
  }
  
  .viewer-video {
      max-width: 80%;
      max-height: 80%;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8);
      outline: none;
      border-radius: 8px;
      transition: transform 0.2s ease-out;
  }
  
  .viewer-audio-card {
      background: rgba(31, 40, 51, 0.45);
      backdrop-filter: blur(16px);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 16px;
      padding: 40px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      width: 400px;
      box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
      text-align: center;
  }
  
  :global(.audio-card-icon) {
      color: #66fcf1;
      margin-bottom: 20px;
      filter: drop-shadow(0 0 8px rgba(102, 252, 241, 0.3));
  }
  
  .viewer-audio-card h3 {
      font-size: 1.1rem;
      color: #ffffff;
      margin: 0 0 8px 0;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
  }
  
  .audio-meta {
      font-size: 0.8rem;
      color: #c5c6c7;
      margin: 0 0 24px 0;
  }
  
  .viewer-audio {
      width: 100%;
      outline: none;
  }
</style>
