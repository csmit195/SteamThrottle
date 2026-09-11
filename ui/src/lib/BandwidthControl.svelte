<script lang="ts">
  import {
    MAX_MEGABYTES_PER_SECOND,
    MIN_MEGABYTES_PER_SECOND,
    bandwidthFillPercent,
    draggedMegabytesPerSecond,
    megabytesToBytes,
  } from "./bandwidth-control";

  let {
    value,
    onchange,
    oninteractionstart,
    oncommit,
  }: {
    value: number;
    onchange: (bytesPerSecond: number) => void;
    oninteractionstart: () => void;
    oncommit: () => void;
  } = $props();

  let input: HTMLInputElement;
  let dragStartX = 0;
  let dragStartMegabytes = 0;
  let dragControlWidth = 1;
  let activePointer: number | null = null;
  let suppressClick = false;
  let pointerDown = $state(false);
  let dragging = $state(false);
  let megabytesPerSecond = $derived(value / 1_000_000);
  let fill = $derived(`${bandwidthFillPercent(megabytesPerSecond)}%`);

  function enterValue(event: Event) {
    const entered = Number((event.currentTarget as HTMLInputElement).value);
    if (Number.isFinite(entered)) onchange(megabytesToBytes(entered));
  }

  function startDrag(event: PointerEvent) {
    if (event.button !== 0) return;
    activePointer = event.pointerId;
    oninteractionstart();
    dragStartX = event.clientX;
    dragStartMegabytes = megabytesPerSecond;
    dragControlWidth =
      (event.currentTarget as HTMLElement).closest<HTMLElement>(".bandwidth-box")?.clientWidth ?? 1;
    pointerDown = true;
    dragging = false;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function drag(event: PointerEvent) {
    if (activePointer !== event.pointerId) return;
    const horizontalPixels = event.clientX - dragStartX;
    if (Math.abs(horizontalPixels) < 2) return;
    dragging = true;
    event.preventDefault();
    onchange(
      megabytesToBytes(
        draggedMegabytesPerSecond(dragStartMegabytes, horizontalPixels, dragControlWidth),
      ),
    );
  }

  function endDrag(event: PointerEvent) {
    if (activePointer !== event.pointerId) return;
    const target = event.currentTarget as HTMLElement;
    if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId);
    activePointer = null;
    if (dragging) {
      suppressClick = true;
      input.blur();
      window.getSelection()?.removeAllRanges();
      oncommit();
    }
    pointerDown = false;
    dragging = false;
  }

  function finishClick(event: MouseEvent) {
    if (!suppressClick) return;
    event.preventDefault();
    suppressClick = false;
    input.blur();
  }

  function finishKeyboardEntry(event: KeyboardEvent) {
    if (event.key === "Enter") input.blur();
  }
</script>

<label class:armed={pointerDown} class:dragging class="bandwidth-box" style:--fill={fill}>
  <span class="value-field">
    <span class="value-sizer" aria-hidden="true">{megabytesPerSecond}</span>
    <input
      bind:this={input}
      type="number"
      min={MIN_MEGABYTES_PER_SECOND}
      max={MAX_MEGABYTES_PER_SECOND}
      step="0.5"
      value={megabytesPerSecond}
      oninput={enterValue}
      onfocus={oninteractionstart}
      onblur={oncommit}
      onkeydown={finishKeyboardEntry}
      onpointerdown={startDrag}
      onpointermove={drag}
      onpointerup={endDrag}
      onpointercancel={endDrag}
      onclick={finishClick}
      aria-label="Gameplay bandwidth limit in megabytes per second; drag left or right to adjust"
    />
  </span>
  <span class="unit">MB/s</span>
</label>

<style>
  .bandwidth-box {
    position: relative;
    isolation: isolate;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    width: fit-content;
    min-width: 72px;
    height: 28px;
    box-sizing: border-box;
    padding: 0 7px;
    overflow: hidden;
    border: 1px solid #34363d;
    border-radius: 4px;
    background: #121316;
    box-shadow: inset 0 1px 2px #0006;
    cursor: col-resize;
    touch-action: none;
  }
  .bandwidth-box::before {
    position: absolute;
    z-index: -1;
    inset: 0 auto 0 0;
    width: var(--fill);
    background: linear-gradient(90deg, #4651be, #5865d8);
    content: "";
    opacity: 0.72;
    pointer-events: none;
  }
  .bandwidth-box:hover {
    border-color: #4d5059;
  }
  .bandwidth-box:focus-within {
    border-color: #7480f0;
    box-shadow:
      0 0 0 2px #18191d,
      0 0 0 3px #6571eb;
  }
  .bandwidth-box.dragging {
    border-color: #8992ef;
    box-shadow:
      0 0 0 2px #18191d,
      0 0 0 3px #6571eb;
  }
  .bandwidth-box.armed,
  .bandwidth-box.armed input,
  .bandwidth-box.dragging,
  .bandwidth-box.dragging input {
    cursor: grabbing;
  }
  .value-field {
    position: relative;
    z-index: 1;
    display: inline-grid;
    min-width: 2ch;
  }
  .value-sizer,
  input {
    grid-area: 1 / 1;
    min-width: 0;
    padding: 0 1px;
    font-weight: 600;
    text-align: right;
  }
  .value-sizer {
    visibility: hidden;
    white-space: pre;
  }
  input {
    position: relative;
    width: 100%;
    height: 26px;
    box-sizing: border-box;
    border: 0;
    outline: 0;
    appearance: textfield;
    color: #fff;
    background: transparent;
    font-weight: 600;
    text-align: right;
    cursor: col-resize;
  }
  .bandwidth-box:focus-within:not(.armed):not(.dragging) input {
    cursor: text;
  }
  input::-webkit-inner-spin-button,
  input::-webkit-outer-spin-button {
    margin: 0;
    appearance: none;
  }
  .unit {
    position: relative;
    z-index: 1;
    padding: 0;
    color: #d2d4d8;
    font-size: 9px;
    white-space: nowrap;
    pointer-events: none;
  }
</style>
