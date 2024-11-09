import { ReactNode, useEffect, useRef, useState } from "react";
import { FolderViews } from "../types";
import { GitBranch, LucideIcon, MessageCircle } from "lucide-react";
import { useStore } from "../Store";

interface ViewOption {
  value: FolderViews;
  icon: LucideIcon;
}

const viewOptions: ViewOption[] = [
  { value: "chat", icon: MessageCircle },
  // { value: "browser", icon: Folder },
  { value: "tree", icon: GitBranch },
];

interface ToggleGroupItemProps {
  value: FolderViews;
  isSelected: boolean;
  onClick: (value: FolderViews) => void;
  children: ReactNode;
}

function ToggleGroupItem({
  value,
  isSelected,
  onClick,
  children,
}: ToggleGroupItemProps) {
  return (
    <button
      onClick={() => onClick(value)}
      className={`p-2 z-10 transition-colors duration-200 ${
        isSelected ? "text-white" : "text-gray-600 hover:text-gray-800"
      }`}
      aria-label={`${value} view`}
    >
      {children}
    </button>
  );
}

// toggle for selecting view inside of folder
export function ViewToggle() {
  const [folderView, setFolderView] = useStore((state) => [state.folderView, state.setFolderView]);
  const [indicatorStyle, setIndicatorStyle] = useState<React.CSSProperties>({});
  const containerRef = useRef<HTMLDivElement>(null);

  const handleViewChange = (newView: FolderViews) => {
    setFolderView(newView);
  };

  useEffect(() => {
    if (containerRef.current) {
      const activeButton = containerRef.current.querySelector(
        `button[aria-label="${folderView} view"]`,
      ) as HTMLButtonElement | null;
      if (activeButton) {
        const { offsetLeft, offsetWidth } = activeButton;
        setIndicatorStyle({
          left: `${offsetLeft}px`,
          width: `${offsetWidth}px`,
        });
      }
    }
  }, [folderView]);

  return (
    <div
      className="relative inline-flex bg-white rounded-full border-2 border-gray-200"
      ref={containerRef}
    >
      <div
        className="absolute top-0 bottom-0 bg-orange-500 rounded-full transition-all duration-300 ease-in-out"
        style={indicatorStyle}
      ></div>
      {viewOptions.map(({ value, icon: Icon }) => (
        <ToggleGroupItem
          key={value}
          value={value}
          isSelected={folderView === value}
          onClick={handleViewChange}
        >
          <Icon className="h-5 w-5" />
        </ToggleGroupItem>
      ))}
    </div>
  );
}
