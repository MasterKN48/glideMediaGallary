<script>
  import { onMount, onDestroy } from "svelte";
  import Thumbnail from "./Thumbnail.svelte";

  let { mediaItems = [], viewMode = "day", onSelectItem } = $props();

  let containerEl = $state(null);
  let containerWidth = $state(800);
  let containerHeight = $state(600);
  let scrollTop = $state(0);

  // Constants based on view mode
  let colWidth = $derived(
    viewMode === "day" ? 150 : viewMode === "month" ? 100 : 75
  );
  let rowHeight = $derived(
    viewMode === "day" ? 150 : viewMode === "month" ? 100 : 75
  );
  let gap = 12;

  // Calculate dynamic column count
  let colCount = $derived(
    Math.max(1, Math.floor((containerWidth + gap) / (colWidth + gap)))
  );

  // Group and sort media
  let groupedMedia = $derived(groupMedia(mediaItems, viewMode));

  // Flatten into rows with absolute vertical positions
  let virtualRows = $derived(calculateVirtualRows(groupedMedia, colCount, rowHeight, gap));

  // Total height of the virtual grid
  let totalHeight = $derived(
    virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].top + virtualRows[virtualRows.length - 1].height : 0
  );

  // Filter rows currently visible in the viewport (plus buffer)
  let visibleRows = $derived.by(() => {
    const buffer = 1200; // 1200px offscreen buffer above and below
    const startY = Math.max(0, scrollTop - buffer);
    const endY = scrollTop + containerHeight + buffer;
    
    return virtualRows.filter(
      row => row.top + row.height >= startY && row.top <= endY
    );
  });

  // Group media by date based on mode
  function groupMedia(items, mode) {
    const groups = {};
    
    // Sort items by creation time descending (newest first)
    const sorted = [...items].sort((a, b) => b.created_time - a.created_time);
    
    for (const item of sorted) {
      const date = new Date(item.created_time * 1000);
      let key = "";
      if (mode === "day") {
        key = date.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' });
      } else if (mode === "month") {
        key = date.toLocaleDateString(undefined, { year: 'numeric', month: 'long' });
      } else {
        key = date.toLocaleDateString(undefined, { year: 'numeric' });
      }
      
      if (!groups[key]) {
        groups[key] = [];
      }
      groups[key].push(item);
    }
    
    return Object.keys(groups).map(title => ({
      title,
      items: groups[title]
    }));
  }

  // Calculate coordinates of headers and grid rows
  function calculateVirtualRows(groups, cols, rowH, itemGap) {
    const rows = [];
    let currentTop = 0;
    
    for (let gIndex = 0; gIndex < groups.length; gIndex++) {
      const group = groups[gIndex];
      
      // Header Row
      const headerHeight = 45;
      rows.push({
        id: `h-${gIndex}`,
        type: "header",
        title: group.title,
        top: currentTop,
        height: headerHeight
      });
      currentTop += headerHeight;
      
      // Media Rows within this group
      const items = group.items;
      for (let i = 0; i < items.length; i += cols) {
        const rowItems = items.slice(i, i + cols);
        rows.push({
          id: `m-${gIndex}-${i}`,
          type: "media",
          items: rowItems,
          top: currentTop,
          height: rowH
        });
        currentTop += rowH + itemGap;
      }
      // Add extra padding after a section
      currentTop += 15;
    }
    
    return rows;
  }

  let resizeObserver;

  onMount(() => {
    if (containerEl) {
      containerWidth = containerEl.clientWidth;
      containerHeight = containerEl.clientHeight;
      
      resizeObserver = new ResizeObserver(entries => {
        for (let entry of entries) {
          containerWidth = entry.contentRect.width;
          containerHeight = entry.contentRect.height;
        }
      });
      resizeObserver.observe(containerEl);
    }
  });

  onDestroy(() => {
    if (resizeObserver && containerEl) {
      resizeObserver.disconnect();
    }
  });

  function handleScroll(e) {
    scrollTop = e.target.scrollTop;
  }
</script>

<div 
  bind:this={containerEl} 
  class="virtual-grid-container" 
  onscroll={handleScroll}
>
  <div class="scroll-filler" style="height: {totalHeight}px;">
    {#each visibleRows as row (row.id)}
      {#if row.type === "header"}
        <div 
          class="grid-section-header" 
          style="transform: translateY({row.top}px); height: {row.height}px;"
        >
          <h2>{row.title}</h2>
          <span class="count-badge">
            {groupedMedia.find(g => g.title === row.title)?.items.length || 0} items
          </span>
        </div>
      {:else}
        <div 
          class="grid-media-row" 
          style="transform: translateY({row.top}px); height: {row.height}px; gap: {gap}px;"
        >
          {#each row.items as item}
            <button 
              class="grid-item-wrapper" 
              style="width: {colWidth}px; height: {rowHeight}px;"
              onclick={() => onSelectItem(item)}
            >
              <Thumbnail {item} />
            </button>
          {/each}
        </div>
      {/if}
    {/each}
  </div>
  
  {#if mediaItems.length === 0}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" class="empty-icon"><path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/></svg>
      <p>No media files found. Add a folder to scan and index your files.</p>
    </div>
  {/if}
</div>

<style>
  .virtual-grid-container {
      flex: 1;
      height: 100%;
      overflow-y: auto;
      position: relative;
      padding: 20px;
      scroll-behavior: smooth;
      /* Custom elegant scrollbar */
      scrollbar-width: thin;
      scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  }
  .virtual-grid-container::-webkit-scrollbar {
      width: 6px;
  }
  .virtual-grid-container::-webkit-scrollbar-track {
      background: var(--scrollbar-track);
  }
  .virtual-grid-container::-webkit-scrollbar-thumb {
      background-color: var(--scrollbar-thumb);
      border-radius: 3px;
  }
  .virtual-grid-container::-webkit-scrollbar-thumb:hover {
      background-color: var(--scrollbar-thumb-hover);
  }
  
  .scroll-filler {
      width: 100%;
      position: relative;
  }
  
  .grid-section-header {
      position: absolute;
      left: 0;
      right: 0;
      display: flex;
      align-items: center;
      justify-content: space-between;
      border-bottom: 1px solid var(--surface-border-elevated);
      padding-bottom: 6px;
      margin-bottom: 12px;
      user-select: none;
      z-index: 5;
  }
  
  .grid-section-header h2 {
      font-size: 1.15rem;
      font-weight: 600;
      color: var(--text-primary);
      margin: 0;
      text-shadow: var(--shadow-text);
  }
  
  .count-badge {
      font-size: 0.8rem;
      color: var(--accent);
      background: var(--accent-subtle);
      padding: 2px 8px;
      border-radius: 12px;
      border: 1px solid var(--accent-border);
  }
  
  .grid-media-row {
      position: absolute;
      left: 0;
      right: 0;
      display: flex;
      flex-direction: row;
      align-items: center;
  }
  
  .grid-item-wrapper {
      background: none;
      border: none;
      padding: 0;
      margin: 0;
      cursor: pointer;
      outline: none;
  }
  
  .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 70%;
      color: var(--text-subtle);
      text-align: center;
      padding: 20px;
  }
  .empty-icon {
      width: 64px;
      height: 64px;
      fill: var(--accent-fill);
      margin-bottom: 16px;
  }
  .empty-state p {
      font-size: 0.95rem;
      max-width: 400px;
      line-height: 1.5;
  }
</style>
