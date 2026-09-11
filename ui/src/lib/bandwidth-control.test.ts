import { describe, expect, it } from "vitest";
import {
  MAX_MEGABYTES_PER_SECOND,
  MIN_MEGABYTES_PER_SECOND,
  bandwidthFillPercent,
  draggedMegabytesPerSecond,
  megabytesToBytes,
} from "./bandwidth-control";

describe("bandwidth control", () => {
  it("fills the textbox progress overlay from its minimum to maximum", () => {
    expect(bandwidthFillPercent(MIN_MEGABYTES_PER_SECOND)).toBe(0);
    expect(bandwidthFillPercent(MAX_MEGABYTES_PER_SECOND)).toBe(100);
    expect(bandwidthFillPercent(1_000)).toBe(100);
  });

  it("converts decimal megabytes to whole bytes", () => {
    expect(megabytesToBytes(10.5)).toBe(10_500_000);
  });

  it("maps one textbox width to the full range while snapping to 0.5 MB/s", () => {
    expect(draggedMegabytesPerSecond(10, 92, 92)).toBe(125);
    expect(draggedMegabytesPerSecond(100, -92, 92)).toBe(MIN_MEGABYTES_PER_SECOND);
    expect(draggedMegabytesPerSecond(10, 1, 100) % 0.5).toBe(0);
  });
});
