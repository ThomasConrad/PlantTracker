/**
 * Touch gesture utilities for mobile interactions.
 * Ported from pop-booking's mobile calendar implementation.
 */

/**
 * Creates a requestAnimationFrame-throttled transform scheduler.
 * Batches rapid updates to a single rAF, always using the latest value.
 */
export function makeTransformScheduler(apply: (value: number) => void) {
  let frame = 0;
  let latest = 0;

  function schedule(value: number) {
    latest = value;
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      apply(latest);
    });
  }

  schedule.cancel = () => {
    if (!frame) return;
    cancelAnimationFrame(frame);
    frame = 0;
  };

  return schedule;
}

/**
 * Tracks touch velocity over time for fling-based gestures.
 */
export class VelocityTracker {
  private lastPos = 0;
  private lastTime = 0;
  public velocity = 0;

  start(pos: number) {
    this.lastPos = pos;
    this.lastTime = Date.now();
    this.velocity = 0;
  }

  update(pos: number) {
    const now = Date.now();
    const dt = now - this.lastTime;
    if (dt > 0) {
      this.velocity = (pos - this.lastPos) / dt;
    }
    this.lastPos = pos;
    this.lastTime = now;
  }
}

/** Apply translate3d(Xpx, 0, 0) for GPU-accelerated horizontal movement */
export function translateX(el: HTMLElement, x: number) {
  el.style.transform = `translate3d(${x}px, 0, 0)`;
}

/** Apply translate3d(0, Ypx, 0) for GPU-accelerated vertical movement */
export function translateY(el: HTMLElement, y: number) {
  el.style.transform = `translate3d(0, ${y}px, 0)`;
}

/** Standard easing curve for snap animations */
export const SNAP_EASING = 'cubic-bezier(0.32, 0.72, 0, 1)';
export const SNAP_DURATION = '0.35s';
export const SNAP_TRANSITION = `transform ${SNAP_DURATION} ${SNAP_EASING}`;

/** Fling velocity threshold (px/ms) */
export const FLING_THRESHOLD = 0.3;

/** Swipe distance threshold as fraction of container width */
export const SWIPE_FRACTION = 0.25;

/**
 * Determines swipe direction from velocity and displacement.
 * Returns -1 (prev), 0 (snap back), or +1 (next).
 */
export function getSwipeDirection(
  velocity: number,
  displacement: number,
  containerWidth: number,
): -1 | 0 | 1 {
  if (velocity > FLING_THRESHOLD || displacement > containerWidth * SWIPE_FRACTION) return -1;
  if (velocity < -FLING_THRESHOLD || displacement < -containerWidth * SWIPE_FRACTION) return 1;
  return 0;
}

/**
 * Detects primary touch direction from accumulated delta.
 */
export function detectDirection(
  dx: number,
  dy: number,
  threshold = 8,
): 'horizontal' | 'vertical' | null {
  if (Math.abs(dx) > threshold || Math.abs(dy) > threshold) {
    return Math.abs(dx) > Math.abs(dy) ? 'horizontal' : 'vertical';
  }
  return null;
}
