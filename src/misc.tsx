import {
  DragDropEvent,
  getCurrentWindow,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { cn } from "./lib/utils";

export function extractFilename(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || "";
}

export function extractFileExtension(path: string): string {
  const parts = path.split(/\./);
  return parts[parts.length - 1] || "";
}

export function truncateAttachmentThing(name: string): string {
  if (name.length > 30) return name.slice(0, 30) + "...";
  return name;
}

export function truncateAttachmentTitle(name: string): string {
  if (name.length > 45) return name.slice(0, 45) + "...";
  return name;
}

// function createDebugDot(x: number, y: number, color = 'red', size = 10, duration = 5) {
//   const dot = document.createElement('div');
//   dot.style.position = 'absolute';
//   dot.style.left = `${x - size / 2}px`;
//   dot.style.top = `${y - size / 2}px`;
//   dot.style.width = `${size}px`;
//   dot.style.height = `${size}px`;
//   dot.style.backgroundColor = color;
//   dot.style.borderRadius = '50%';
//   dot.style.zIndex = '9999';
//   dot.style.pointerEvents = 'none';

//   document.body.appendChild(dot);

//   if (duration > 0) {
//     setTimeout(() => {
//       document.body.removeChild(dot);
//     }, duration);
//   } else {
//     document.body.removeChild(dot);
//   }

//   return dot;
// }

// just checks whether given point is within element
export function checkWithin(point: PhysicalPosition, node: HTMLElement) {
  let { x, y } = point;
  // TODO tauri pointer position seems to be lower than actual spot
  // y -= 20;
  // createDebugDot(x, y);
  const elemAtPoint = document.elementFromPoint(x, y);
  return node === elemAtPoint;
}

export function checkWithinWithChildren(
  point: PhysicalPosition,
  node: HTMLElement
) {
  let { x, y } = point;
  // TODO tauri pointer position seems to be lower than actual spot
  // y -= 20;
  // createDebugDot(x, y);
  const elemAtPoint = document.elementFromPoint(x, y);
  return node.contains(elemAtPoint);
}

export function useFocusedInParent(
  parent: React.RefObject<HTMLElement | null>
): boolean {
  const [focused, setFocused] = useState<boolean>(false);

  useEffect(() => {
    function handleEvent(
      e: Event,
      ifChildIsTarget: boolean,
      cb: (v: boolean) => void
    ) {
      if (parent.current && e.target && e.target instanceof Element) {
        if (parent.current.contains(e.target)) cb(ifChildIsTarget);
      }
    }

    const focusIn = (e: Event) => handleEvent(e, true, setFocused);
    const focusOut = (e: Event) => handleEvent(e, false, setFocused);
    document.addEventListener("focusin", focusIn);
    document.addEventListener("focusout", focusOut);

    return () => {
      document.removeEventListener("focusin", focusIn);
      document.removeEventListener("focusout", focusOut);
    };
  }, [parent]);

  return focused;
}

export function useMousedInParent(
  parent: React.RefObject<HTMLElement | null>
): boolean {
  const [moused, setMoused] = useState<boolean>(false);

  useEffect(() => {
    function handleEvent(
      e: Event,
      ifChildIsTarget: boolean,
      cb: (v: boolean) => void
    ) {
      if (parent.current && e.target && e.target instanceof Element) {
        if (parent.current.contains(e.target)) cb(ifChildIsTarget);
      }
    }

    const mouseOver = (e: Event) => handleEvent(e, true, setMoused);
    const mouseOff = (e: Event) => handleEvent(e, false, setMoused);
    parent.current?.addEventListener("mouseenter", mouseOver);
    parent.current?.addEventListener("mouseleave", mouseOff);

    return () => {
      parent.current?.removeEventListener("mouseenter", mouseOver);
      parent.current?.removeEventListener("mouseleave", mouseOff);
    };
  }, [parent]);

  return moused;
}

export function useMousedOrFocusedInParent(
  parent: React.RefObject<HTMLElement | null>
): boolean {
  const moused = useMousedInParent(parent);
  const focused = useFocusedInParent(parent);
  return moused || focused;
}

export function useTauriDragInParent(
  parent: React.RefObject<HTMLElement | null>
): boolean {
  const [draggedOver, setDraggedOver] = useState(false);
  const tauriCleanup = useRef<(() => void) | null>(null);

  useEffect(() => {
    (async () => {
      tauriCleanup.current = await getCurrentWindow().onDragDropEvent((e) => {
        if (
          (e.payload.type == "enter" || e.payload.type == "over") &&
          parent.current
        ) {
          if (checkWithinWithChildren(e.payload.position, parent.current)) {
            setDraggedOver(true);
          }
        } else {
          setDraggedOver(false);
        }
      });
    })();

    return () => {
      tauriCleanup.current?.();
    };
  }, []);

  return draggedOver;
}

export function useHandleTauriDrag(handler: (e: DragDropEvent) => void) {
  const tauriCleanup = useRef<(() => void) | null>(null);

  useEffect(() => {
    (async () => {
      tauriCleanup.current = await getCurrentWindow().onDragDropEvent((e) => {
        handler(e.payload);
      }
      );
    })();

    return () => {
      tauriCleanup.current?.();
    };
  }, []);
}

// https://github.com/shadcn-ui/ui/discussions/1694#discussioncomment-7477119
export function LoadingSpinnerBlack({ className }: { className?: any}) {
  return (
    <svg
      // xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={cn("animate-spin", className)}
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

export function LoadingSpinnerWhite({ className }: { className?: any}) {
  return (
    <svg
      // xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="white"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={cn("animate-spin", className)}
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

export function fallbackRender({
  error,
  // resetErrorBoundary,
}: {
  error: any;
  // resetErrorBoundary: any;
}) {
  // Call resetErrorBoundary() to reset the error boundary and retry the render.

  return (
    <div role="alert" className="m-8 bg-inherit">
      <p>Something went wrong:</p>
      <pre style={{ color: "red" }}>{error.message}</pre>
      <p>You can try opening the developer tools to see if there are any helpful error messages.</p>
    </div>
  );
}
