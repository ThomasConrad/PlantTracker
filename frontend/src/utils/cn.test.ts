import { describe, it, expect } from "vitest";
import { cn } from "./cn";

describe("cn", () => {
  it("joins multiple class strings", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("filters out falsy values", () => {
    expect(cn("foo", undefined, null, false, "bar")).toBe("foo bar");
  });

  it("returns empty string when all values are falsy", () => {
    expect(cn(undefined, null, false)).toBe("");
  });

  it("returns single class", () => {
    expect(cn("only")).toBe("only");
  });

  it("handles empty string (falsy)", () => {
    expect(cn("a", "", "b")).toBe("a b");
  });
});
