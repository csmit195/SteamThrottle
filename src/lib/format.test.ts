import { describe, expect, it } from "vitest";
import { actionLabel, stateLabel } from "./format";

describe("status formatting", () => {
  it("labels the decimal 10 MB/s limit without confusing it with megabits", () => {
    expect(actionLabel({ kind: "limit", bytesPerSecond: 10_000_000 })).toBe("Limited to 10.0 MB/s");
  });

  it("distinguishes preparation, combat, and death", () => {
    expect(
      stateLabel({
        clientPhase: "inGame",
        mode: "arena",
        arenaPhase: "preparation",
        life: "alive",
        confidence: 100,
      }),
    ).toBe("Arena preparation");
    expect(
      stateLabel({
        clientPhase: "inGame",
        mode: "arena",
        arenaPhase: "combat",
        life: "dead",
        confidence: 100,
      }),
    ).toBe("Arena combat · dead");
  });
});
