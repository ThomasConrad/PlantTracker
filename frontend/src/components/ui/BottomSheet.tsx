import {
  Component,
  JSX,
  Show,
  createSignal,
  createEffect,
  onCleanup,
} from "solid-js";
import {
  makeTransformScheduler,
  VelocityTracker,
  translateY,
  SNAP_TRANSITION,
  FLING_THRESHOLD,
  detectDirection,
} from "@/utils/touch";

export interface BottomSheetProps {
  /** Whether the sheet is visible */
  open: boolean;
  /** Called when the user dismisses the sheet (drag down past threshold) */
  onDismiss: () => void;
  /** Optional: called when sheet open/close state changes */
  onOpenChange?: (isFullOpen: boolean) => void;
  /** Header content (rendered above the scrollable area) */
  header?: JSX.Element;
  /** Custom backdrop content (e.g. an image). Replaces the default dim overlay. */
  backdrop?: JSX.Element;
  /** Main scrollable content */
  children: JSX.Element;
  /** Height from top when fully expanded (px). Default 56. */
  fullTopOffset?: number;
  /** Fraction of viewport for mid (collapsed) position. Default 0.55 */
  midFraction?: number;
  /** Whether sheet can be dismissed by dragging below mid. Default true. */
  dismissible?: boolean;
  /** Extra class on the overlay container */
  class?: string;
}

export const BottomSheet: Component<BottomSheetProps> = (props) => {
  const FULL_TOP = () => props.fullTopOffset ?? 56;
  const MID_FRACTION = () => props.midFraction ?? 0.55;

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_sheetOpen, setSheetOpen] = createSignal(false);
  const [visible, setVisible] = createSignal(false);

  let overlayRef: HTMLDivElement | undefined;
  let scrollRef: HTMLDivElement | undefined;
  let backdropRef: HTMLDivElement | undefined;
  let cleanup: (() => void) | undefined;

  const viewportH = () => window.visualViewport?.height || window.innerHeight;
  const getFullTop = () => FULL_TOP();
  const getMidTop = () => viewportH() * MID_FRACTION();
  const getDismissTop = () => viewportH() * 0.85;

  const clampOverlayTop = (y: number) =>
    Math.max(getFullTop(), Math.min(getDismissTop(), y));

  const setupGestures = () => {
    if (!overlayRef || !scrollRef) return;

    const overlay = overlayRef;
    const scrollEl = scrollRef;

    const scheduleY = makeTransformScheduler((y) => {
      overlay.style.height = Math.max(120, viewportH() - FULL_TOP()) + "px";
      translateY(overlay, y);
    });

    const velocity = new VelocityTracker();
    let dragging = false;
    let pending = false;
    let direction: "horizontal" | "vertical" | null = null;
    let startY = 0;
    let startX = 0;
    let startTop = 0;
    let latestTop = 0;

    function getLiveTop() {
      return overlay.getBoundingClientRect().top;
    }

    function snapOverlay(vel: number, currentTop: number) {
      overlay.style.transition = SNAP_TRANSITION;
      const midTop = getMidTop();
      const fullTop = getFullTop();
      const midPoint = (fullTop + midTop) / 2;
      const dismissThreshold = (midTop + getDismissTop()) / 2;

      // Dismiss if dragged far down or flung down fast
      if (
        (props.dismissible !== false) &&
        (vel > FLING_THRESHOLD * 1.5 || currentTop > dismissThreshold)
      ) {
        // Animate out then dismiss
        translateY(overlay, viewportH());
        setTimeout(() => props.onDismiss(), 350);
        return;
      }

      let snapTo: number;
      if (vel < -FLING_THRESHOLD) {
        snapTo = fullTop;
      } else if (vel > FLING_THRESHOLD) {
        snapTo = midTop;
      } else {
        snapTo = currentTop < midPoint ? fullTop : midTop;
      }

      overlay.style.height = Math.max(120, viewportH() - FULL_TOP()) + "px";
      translateY(overlay, snapTo);
      const isOpen = snapTo <= fullTop;
      setSheetOpen(isOpen);
      props.onOpenChange?.(isOpen);
    }

    const onTouchStart = (e: TouchEvent) => {
      scheduleY.cancel();
      startX = e.touches[0].clientX;
      startY = e.touches[0].clientY;
      startTop = getLiveTop();
      latestTop = startTop;
      velocity.start(startY);
      dragging = false;
      pending = true;
      direction = null;
    };

    const onTouchMove = (e: TouchEvent) => {
      if (!pending) return;
      const x = e.touches[0].clientX;
      const y = e.touches[0].clientY;
      const dx = x - startX;
      const dy = y - startY;

      if (!direction) {
        direction = detectDirection(dx, dy);
      }
      if (direction === "horizontal") return;
      if (direction !== "vertical") return;

      // Allow native scroll inside content when sheet is fully open
      if (!dragging && (e.target as HTMLElement)?.closest(".bottom-sheet-scroll")) {
        const maxScroll = scrollEl.scrollHeight - scrollEl.clientHeight;
        const atTop = getLiveTop() <= getFullTop() + 2;
        const canScrollUp = scrollEl.scrollTop > 0 && dy > 0;
        const canScrollDown = scrollEl.scrollTop < maxScroll && dy < 0;
        if (atTop && maxScroll > 1 && (canScrollUp || canScrollDown)) {
          startY = y;
          startTop = getLiveTop();
          latestTop = startTop;
          velocity.start(y);
          return;
        }
        if (!atTop || dy > 0) scrollEl.scrollTop = 0;
      }

      if (!dragging) {
        dragging = true;
        overlay.style.transition = "none";
        startY = y;
        startTop = getLiveTop();
        latestTop = startTop;
        velocity.start(y);
      }

      e.preventDefault();
      velocity.update(y);
      latestTop = clampOverlayTop(startTop + (y - startY));
      scheduleY(startTop + (y - startY));
    };

    const onTouchEnd = () => {
      if (!pending) return;
      pending = false;
      if (!dragging) return;
      dragging = false;
      scheduleY.cancel();
      overlay.style.height = Math.max(120, viewportH() - FULL_TOP()) + "px";
      translateY(overlay, latestTop);
      snapOverlay(velocity.velocity, latestTop);
    };

    overlay.addEventListener("touchstart", onTouchStart, { passive: true });
    overlay.addEventListener("touchmove", onTouchMove, { passive: false });
    overlay.addEventListener("touchend", onTouchEnd);
    overlay.addEventListener("touchcancel", onTouchEnd);

    return () => {
      scheduleY.cancel();
      overlay.removeEventListener("touchstart", onTouchStart);
      overlay.removeEventListener("touchmove", onTouchMove);
      overlay.removeEventListener("touchend", onTouchEnd);
      overlay.removeEventListener("touchcancel", onTouchEnd);
    };
  };

  // Animate in when opened
  createEffect(() => {
    if (props.open) {
      setVisible(true);
      // Wait for DOM, then animate from bottom to mid
      requestAnimationFrame(() => {
        if (!overlayRef) return;
        const overlay = overlayRef;
        overlay.style.height =
          Math.max(120, viewportH() - FULL_TOP()) + "px";
        // Start off-screen
        translateY(overlay, viewportH());
        overlay.style.transition = "none";

        requestAnimationFrame(() => {
          overlay.style.transition = SNAP_TRANSITION;
          translateY(overlay, getMidTop());
          setSheetOpen(false);

          // Setup gestures
          cleanup?.();
          cleanup = setupGestures();
        });
      });
    } else {
      // Animate out
      if (overlayRef) {
        overlayRef.style.transition = SNAP_TRANSITION;
        translateY(overlayRef, viewportH());
      }
      // Remove after animation
      setTimeout(() => {
        setVisible(false);
        setSheetOpen(false);
        cleanup?.();
        cleanup = undefined;
      }, 350);
    }
  });

  onCleanup(() => {
    cleanup?.();
  });

  // Dismiss on backdrop tap
  const handleBackdropClick = () => {
    if (props.dismissible !== false) {
      props.onDismiss();
    }
  };

  return (
    <Show when={visible()}>
      <div class="bottom-sheet-container">
        {/* Backdrop: custom content or default dim */}
        <Show
          when={props.backdrop}
          fallback={
            <div
              ref={backdropRef}
              class={`bottom-sheet-backdrop ${props.open ? "bottom-sheet-backdrop-visible" : ""}`}
              onClick={handleBackdropClick}
            />
          }
        >
          <div
            ref={backdropRef}
            class={`bottom-sheet-backdrop-custom ${props.open ? "bottom-sheet-backdrop-custom-visible" : ""}`}
            onClick={handleBackdropClick}
          >
            {props.backdrop}
          </div>
        </Show>

        {/* Sheet */}
        <div
          ref={overlayRef}
          class={`bottom-sheet-panel ${props.class || ""}`}
        >
          {/* Drag handle */}
          <button class="bottom-sheet-handle" aria-label="Drag to resize" />

          {/* Header */}
          <Show when={props.header}>
            <div class="bottom-sheet-header">{props.header}</div>
          </Show>

          {/* Scrollable content */}
          <div class="bottom-sheet-scroll" ref={scrollRef}>
            {props.children}
          </div>
        </div>
      </div>
    </Show>
  );
};
