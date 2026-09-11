export const MIN_MEGABYTES_PER_SECOND = 0.128;
export const MAX_MEGABYTES_PER_SECOND = 125;
export const DRAG_STEP_MEGABYTES_PER_SECOND = 0.5;

export function bandwidthFillPercent(megabytesPerSecond: number): number {
  const fill =
    ((megabytesPerSecond - MIN_MEGABYTES_PER_SECOND) /
      (MAX_MEGABYTES_PER_SECOND - MIN_MEGABYTES_PER_SECOND)) *
    100;
  return Math.min(100, Math.max(0, fill));
}

export function megabytesToBytes(megabytesPerSecond: number): number {
  return Math.round(megabytesPerSecond * 1_000_000);
}

export function draggedMegabytesPerSecond(
  start: number,
  horizontalPixels: number,
  controlWidth: number,
): number {
  const availableSteps = Math.round(
    (MAX_MEGABYTES_PER_SECOND - MIN_MEGABYTES_PER_SECOND) / DRAG_STEP_MEGABYTES_PER_SECOND,
  );
  const steps = Math.round((horizontalPixels / Math.max(controlWidth, 1)) * availableSteps);
  const dragged = start + steps * DRAG_STEP_MEGABYTES_PER_SECOND;
  return Math.min(
    MAX_MEGABYTES_PER_SECOND,
    Math.max(MIN_MEGABYTES_PER_SECOND, Number(dragged.toFixed(3))),
  );
}
