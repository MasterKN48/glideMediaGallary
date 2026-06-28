<script>
  import { onMount, onDestroy } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";

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
  let isFullscreenState = $state(false);

  let rotationStyle = $derived(
    item.orientation === 3 ? "rotate(180deg)" :
    item.orientation === 6 ? "rotate(90deg)" :
    item.orientation === 8 ? "rotate(270deg)" :
    "rotate(0deg)"
  );

  function toggleHTML5Fullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().then(() => {
        isFullscreenState = true;
      }).catch((err) => {
        console.error(`Error attempting to enable fullscreen: ${err.message}`);
      });
    } else {
      document.exitFullscreen().then(() => {
        isFullscreenState = false;
      });
    }
  }

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
          <svg viewBox="0 0 24 24"><path d="M19 13H5v-2h14v2z"/></svg>
        </button>
        <button class="icon-btn" onclick={handleZoomIn} title="Zoom In">
          <svg viewBox="0 0 24 24"><path d="M19 13h-6v6h-2h-6v-2h6V5h2v6h6v2z"/></svg>
        </button>
        <button class="icon-btn" onclick={resetZoom} title="Reset Zoom">
          <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
        </button>
      {/if}
      <button class="icon-btn" onclick={toggleHTML5Fullscreen} title="Toggle Fullscreen (F)">
        {#if isFullscreenState}
          <svg viewBox="0 0 24 24"><path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z"/></svg>
        {:else}
          <svg viewBox="0 0 24 24"><path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/></svg>
        {/if}
      </button>
      <button class="close-btn" onclick={onClose} title="Close (Esc)">
        <svg viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
      </button>
    </div>
  </div>

  <!-- Navigation Arrows -->
  {#if onPrev}
    <button class="nav-arrow left" onclick={() => { resetZoom(); onPrev(); }} title="Previous">
      <svg viewBox="0 0 24 24"><path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/></svg>
    </button>
  {/if}
  
  {#if onNext}
    <button class="nav-arrow right" onclick={() => { resetZoom(); onNext(); }} title="Next">
      <svg viewBox="0 0 24 24"><path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/></svg>
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
        class="viewer-image {item.orientation === 6 || item.orientation === 8 ? 'rotated-90-deg' : ''}"
        style="transform: scale({zoom}) translate({panX}px, {panY}px) {rotationStyle}; cursor: {zoom > 1 ? 'grab' : 'default'}"
        onmousedown={handleMouseDown}
        draggable="false"
      />
    {:else if item.media_type === "video"}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video 
        bind:this={videoEl}
        src={mediaSrc}
        class="viewer-video"
        controls
        autoplay
        onplay={() => isPlaying = true}
        onpause={() => isPlaying = false}
      ></video>
    {:else if item.media_type === "audio"}
      <div class="viewer-audio-card">
        <svg viewBox="0 0 24 24" class="audio-card-icon"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
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
      right: 0;
      bottom: 0;
      background: rgba(10, 10, 15, 0.9);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
      z-index: 1000;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      user-select: none;
  }
  
  .player-top-bar {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 60px;
      background: rgba(31, 40, 51, 0.35);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 20px;
      z-index: 1010;
  }
  
  .media-title {
      display: flex;
      flex-direction: column;
  }
  .file-name {
      font-size: 0.95rem;
      font-weight: 500;
      color: #ffffff;
      max-width: 300px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
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
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid rgba(255, 255, 255, 0.1);
      width: 36px;
      height: 36px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #c5c6c7;
      cursor: pointer;
      transition: all 0.2s;
  }
  .icon-btn svg, .close-btn svg {
      width: 18px;
      height: 18px;
      fill: currentColor;
  }
  .icon-btn:hover {
      color: #66fcf1;
      border-color: rgba(102, 252, 241, 0.4);
      background: rgba(102, 252, 241, 0.1);
  }
  .close-btn:hover {
      color: #ff5f56;
      border-color: rgba(255, 95, 86, 0.4);
      background: rgba(255, 95, 86, 0.1);
  }
  
  .nav-arrow {
      position: absolute;
      top: 50%;
      transform: translateY(-50%);
      background: rgba(31, 40, 51, 0.5);
      border: 1px solid rgba(255, 255, 255, 0.08);
      width: 50px;
      height: 50px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #ffffff;
      cursor: pointer;
      z-index: 1005;
      transition: all 0.2s;
      opacity: 0.6;
  }
  .nav-arrow:hover {
      opacity: 1;
      color: #66fcf1;
      border-color: rgba(102, 252, 241, 0.4);
      background: rgba(102, 252, 241, 0.15);
  }
  .nav-arrow svg {
      width: 24px;
      height: 24px;
      fill: currentColor;
  }
  .nav-arrow.left {
      left: 30px;
  }
  .nav-arrow.right {
      right: 30px;
  }
  
  .media-content-container {
      width: 100%;
      height: 100%;
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
  .audio-card-icon {
      width: 64px;
      height: 64px;
      fill: #66fcf1;
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
