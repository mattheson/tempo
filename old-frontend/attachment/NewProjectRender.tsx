import { DragDropEvent } from "@tauri-apps/api/window";
import {
  checkWithinWithChildren,
  extractFilename,
  useHandleTauriDrag,
} from "../misc";
import { useStore } from "../Store";
import { useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Plus } from "lucide-react";
import { XButton } from "@/XButton";

export function NewProjectRender({
  renderPath,
  setRenderPath,
}: {
  renderPath: string | null;
  setRenderPath: (path: string | null) => void;
}) {
  const [addError] = useStore((state) => [state.addError]);

  const [isHoveringRender, setIsHoveringRender] = useState(false);

  const addRenderRef = useRef<HTMLDivElement | null>(null);
  const lastDropTime = useRef<number | null>(null);

  useHandleTauriDrag(handler);

  function trySetRenderPath(path: string) {
    if (!setRenderPath) {
      console.warn(`tried to set render path to ${path} without setRenderPath`);
      return;
    }

    setRenderPath(path);
  }

  function handler(event: DragDropEvent) {
    if (!addRenderRef.current) return;

    if (event.type == "leave") {
      setIsHoveringRender(false);
      return;
    }

    const isOver = checkWithinWithChildren(
      event.position,
      addRenderRef.current
    );
    if (!isOver) {
      setIsHoveringRender(false);
      return;
    }

    switch (event.type) {
      case "enter":
      case "over":
        setIsHoveringRender(true);
        break;
      case "drop":
        if (lastDropTime.current && Date.now() - lastDropTime.current < 200)
          return;
        lastDropTime.current = Date.now();
        setIsHoveringRender(false);
        if (event.paths.length != 1) {
          addError("Please drag and drop only one render at most.");
        } else {
          trySetRenderPath(event.paths[0]);
        }
    }
  }

  return renderPath ? (
    <div className="flex-col items-center p-6 shadow-sm rounded border border-gray-300 cursor-default relative">
      <XButton onClick={() => setRenderPath(null)} />
      <b>Attached render</b>
      <br />
      {extractFilename(renderPath)}
    </div>
  ) : (
    <div
      ref={addRenderRef}
      className={`flex flex-col items-center p-6 shadow-sm rounded border border-gray-300 duration-300 select-none cursor-default hover:shadow-lg ${
        isHoveringRender && "bg-blue-50"
      }`}
      onClick={() => {
        if (renderPath || !trySetRenderPath) return;
        (async () => {
          const path = await open({
            multiple: false,
            title: "Select a render to attach",
          });
          if (path) {
            trySetRenderPath(path);
          }
        })();
      }}
    >
      <Plus
        size={24}
        className="text-gray-500 transition-all duration-300 hover:text-gray-700"
      />
      <span className="text-sm text-gray-500 transition-all duration-300 hover:text-gray-700">
        Click or drag a <b>project render</b> here
      </span>
    </div>
  );
}
