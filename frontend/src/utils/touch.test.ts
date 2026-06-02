import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  VelocityTracker,
  getSwipeDirection,
  detectDirection,
  FLING_THRESHOLD,
} from "./touch";

describe("VelocityTracker", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts with zero velocity", () => {
    const tracker = new VelocityTracker();
    tracker.start(100);
    expect(tracker.velocity).toBe(0);
  });

  it("calculates positive velocity for increasing position", () => {
    const tracker = new VelocityTracker();
    tracker.start(0);
    vi.advanceTimersByTime(16);
    tracker.update(48); // 48px in 16ms = 3 px/ms
    expect(tracker.velocity).toBe(3);
  });

  it("calculates negative velocity for decreasing position", () => {
    const tracker = new VelocityTracker();
    tracker.start(100);
    vi.advanceTimersByTime(10);
    tracker.update(50); // -50px in 10ms = -5 px/ms
    expect(tracker.velocity).toBe(-5);
  });
});

describe("getSwipeDirection", () => {
  const containerWidth = 400;

  it("returns -1 (prev) for high positive velocity", () => {
    expect(getSwipeDirection(FLING_THRESHOLD + 0.1, 0, containerWidth)).toBe(
      -1,
    );
  });

  it("returns 1 (next) for high negative velocity", () => {
    expect(getSwipeDirection(-FLING_THRESHOLD - 0.1, 0, containerWidth)).toBe(
      1,
    );
  });

  it("returns -1 for large positive displacement", () => {
    expect(getSwipeDirection(0, containerWidth * 0.3, containerWidth)).toBe(-1);
  });

  it("returns 1 for large negative displacement", () => {
    expect(getSwipeDirection(0, -containerWidth * 0.3, containerWidth)).toBe(1);
  });

  it("returns 0 (snap back) for small displacement and low velocity", () => {
    expect(getSwipeDirection(0.1, 20, containerWidth)).toBe(0);
  });
});

describe("detectDirection", () => {
  it("returns null when both deltas below threshold", () => {
    expect(detectDirection(3, 3)).toBeNull();
  });

  it("returns horizontal when dx > dy and above threshold", () => {
    expect(detectDirection(20, 5)).toBe("horizontal");
  });

  it("returns vertical when dy > dx and above threshold", () => {
    expect(detectDirection(5, 20)).toBe("vertical");
  });

  it("respects custom threshold", () => {
    expect(detectDirection(5, 3, 4)).toBe("horizontal");
    expect(detectDirection(5, 3, 10)).toBeNull();
  });

  it("handles negative values", () => {
    expect(detectDirection(-15, 3)).toBe("horizontal");
    expect(detectDirection(2, -15)).toBe("vertical");
  });
});
