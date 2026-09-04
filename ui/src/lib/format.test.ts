import { describe, expect, it } from "vitest";
import { actionLabel, automationStatusLabel, ruleNameLabel, stateLabel } from "./format";

describe("status formatting", () => {
  it("labels automation state without repeating the control name", () => {
    expect(automationStatusLabel(true)).toBe("Active");
    expect(automationStatusLabel(false)).toBe("Disabled");
  });

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

  it("names the catch-all rule as a fallback", () => {
    expect(ruleNameLabel({ id: "fail-safe", name: "Unknown state" })).toBe("Fallback");
    expect(ruleNameLabel({ id: "arena-combat", name: "Arena combat" })).toBe("Arena combat");
  });
});
