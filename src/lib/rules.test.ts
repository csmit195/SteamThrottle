import { describe, expect, it } from "vitest";
import { reorderRules } from "./rules";

describe("rule reordering", () => {
  it("moves a dragged rule and rewrites priorities", () => {
    const rules = [
      { id: "first", priority: 0 },
      { id: "second", priority: 1 },
      { id: "third", priority: 2 },
    ];

    expect(reorderRules(rules, 0, 2)).toEqual([
      { id: "second", priority: 0 },
      { id: "third", priority: 1 },
      { id: "first", priority: 2 },
    ]);
  });

  it("leaves rules unchanged for an invalid drop", () => {
    const rules = [{ id: "only", priority: 0 }];
    expect(reorderRules(rules, 0, 4)).toEqual(rules);
  });
});
