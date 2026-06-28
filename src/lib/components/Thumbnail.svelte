<script>
  import { onMount } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";

  let { item } = $props();
  let imgSrc = $state("");
  let loading = $state(true);
  let el = $state(null);

  let rotationStyle = $derived(
    item.orientation === 3 ? "transform: rotate(180deg);" :
    item.orientation === 6 ? "transform: rotate(90deg);" :
    item.orientation === 8 ? "transform: rotate(270deg);" :
    ""
  );

  async function loadThumbnail() {
      if (item.media_type !== "image") {
          // Video or audio placeholder
          loading = false;
          return;
      }

      if (item.thumbnail_path) {
          imgSrc = convertFileSrc(item.thumbnail_path);
          loading = false;
          return;
      }

      try {
          // Request thumbnail generation or retrieval from Rust
          const cachedPath = await invoke("get_or_create_thumbnail", { filePath: item.file_path });
          imgSrc = convertFileSrc(cachedPath);
          item.thumbnail_path = cachedPath;
      } catch (err) {
          console.error("Failed to generate thumbnail:", err);
          // Fallback to original image if generation fails
          imgSrc = convertFileSrc(item.file_path);
      } finally {
          loading = false;
      }
  }

  onMount(() => {
      const observer = new IntersectionObserver((entries) => {
          if (entries[0].isIntersecting) {
              loadThumbnail();
              observer.disconnect();
          }
      }, { rootMargin: "200px" });

      if (el) {
          observer.observe(el);
      }

      return () => observer.disconnect();
  });
</script>

<div bind:this={el} class="thumb-container">
  {#if loading}
    <div class="skeleton"></div>
  {:else if item.media_type === "image"}
    <img src={imgSrc} alt={item.filename} class="media-thumb" style={rotationStyle} loading="lazy" />
  {:else if item.media_type === "video"}
    <div class="video-placeholder">
      <svg viewBox="0 0 24 24" class="play-icon"><path d="M8 5v14l11-7z"/></svg>
      <span class="file-name">{item.filename}</span>
    </div>
  {:else}
    <div class="audio-placeholder">
      <svg viewBox="0 0 24 24" class="audio-icon"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
      <span class="file-name">{item.filename}</span>
    </div>
  {/if}
</div>

<style>
  .thumb-container {
      width: 100%;
      height: 100%;
      position: relative;
      background: rgba(255, 255, 255, 0.02);
      border-radius: 8px;
      overflow: hidden;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), box-shadow 0.2s;
  }
  .thumb-container:hover {
      transform: scale(1.03);
      box-shadow: 0 4px 15px rgba(102, 252, 241, 0.3);
  }
  .media-thumb {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
      transition: opacity 0.3s ease;
  }
  .skeleton {
      width: 100%;
      height: 100%;
      background: linear-gradient(90deg, rgba(255,255,255,0.01) 25%, rgba(255,255,255,0.05) 50%, rgba(255,255,255,0.01) 75%);
      background-size: 200% 100%;
      animation: loading 1.5s infinite;
  }
  @keyframes loading {
      0% { background-position: 200% 0; }
      100% { background-position: -200% 0; }
  }
  .video-placeholder, .audio-placeholder {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 100%;
      background: rgba(20, 20, 25, 0.7);
      color: #66fcf1;
      padding: 10px;
      text-align: center;
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 8px;
  }
  .play-icon, .audio-icon {
      width: 36px;
      height: 36px;
      fill: currentColor;
      margin-bottom: 6px;
      opacity: 0.85;
      filter: drop-shadow(0 0 4px rgba(102, 252, 241, 0.4));
  }
  .file-name {
      font-size: 0.7rem;
      max-width: 90%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      color: #a8a8af;
  }
</style>
