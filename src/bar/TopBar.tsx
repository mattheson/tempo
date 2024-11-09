import { useEffect, useRef, useState } from "react";

import {
  ChevronDown,
  Home,
  Folder,
  Globe,
  Plus,
  Hash,
  // User,
  // Undo2,
  Settings,
  Plug,
} from "lucide-react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { ViewToggle } from "./ViewToggle";
import { Input } from "@/components/ui/input";
import { createChannel } from "../commands";
import { extractFilename, LoadingSpinnerBlack } from "../misc";
import { useStore } from "../Store";
import { /*FolderViews, */ getChannelName } from "../types";

export function TopBar() {
  const [
    invokeWithError,

    folder,

    folderData,
    folders,

    view,
    setView,

    folderView,
    // setFolderView,

    setFolder,

    channelUlid,
    setChannel,
    setSettingsOpen,
    setPluginsOpen,
  ] = useStore((state) => [
    state.invokeWithError,

    state.folder,

    state.folderData,
    state.folders!,

    state.view,
    state.setView,

    state.folderView,
    // state.setFolderView,

    state.setFolder,

    state.channelUlid,
    state.setChannel,
    state.setSettingsOpen,
    state.setPluginsOpen,
  ]);

  const [isHovering, setIsHovering] = useState(false);
  const [isFolderDropdownOpen, setIsFolderDropdownOpen] = useState(false);
  const [isChannelDropdownOpen, setIsChannelDropdownOpen] = useState(false);
  const [scrollPosition, setScrollPosition] = useState(0);
  const [isAddingChannel, setIsAddingChannel] = useState(false);
  const [newChannelName, setNewChannelName] = useState("");

  // const [lastFolderView, setLastFolderView] = useState<FolderViews>("chat");

  const elementRef = useRef<HTMLDivElement>(null);

  const anyDropdownOpen = isFolderDropdownOpen || isChannelDropdownOpen;

  useEffect(() => {
    const handleScroll = () => {
      setScrollPosition(window.scrollY);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  // TODO memoize this?

  const isScrolled = scrollPosition !== 0;

  const containerStyles = `
    fixed top-0 left-0 right-0 p-4 flex justify-start 
    transition-all duration-300 z-10
  `;

  const sharedButtonStyles = `
    px-2 pb-2 flex space-x-2 rounded-md transition-all duration-300 
    focus:outline-none focus-visible:ring-offset-0 focus-visible:ring-0 pointer-events-auto
    ${isScrolled ? "bg-white/80 backdrop-blur-sm shadow-sm" : ""}
    ${
      isScrolled && !isHovering && !anyDropdownOpen
        ? "opacity-25"
        : "opacity-100"
    }
  `;

  const buttonStyle = {
    transition:
      "opacity 0.3s ease-in-out, background-color 0.3s ease-in-out, box-shadow 0.3s ease-in-out",
  };

  function handleAddChannelClick(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setIsAddingChannel(true);
  }

  useEffect(() => {
    const checkMouseOver = (event: MouseEvent) => {
      if (elementRef.current) {
        var rect = elementRef.current.getBoundingClientRect();
        rect.height += 20;
        const isOver =
          event.clientX >= rect.left &&
          event.clientX <= rect.right &&
          event.clientY >= rect.top &&
          event.clientY <= rect.bottom;

        setIsHovering(isOver);
      }
    };

    window.addEventListener("mousemove", checkMouseOver);

    return () => {
      window.removeEventListener("mousemove", checkMouseOver);
    };
  }, []);

  function handleAddChannel(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      invokeWithError(createChannel(folder!, newChannelName));
      setNewChannelName("");
      setIsAddingChannel(false);
    }
  }

  function renderFolderDropdown() {
    if (folders == null) {
      return <LoadingSpinnerBlack />;
    } else {
      return (
        <DropdownMenu onOpenChange={setIsFolderDropdownOpen} modal={false}>
          <DropdownMenuTrigger
            asChild
            disabled={Object.keys(folders).length === 0}
          >
            <Button
              variant="ghost"
              className={sharedButtonStyles}
              style={buttonStyle}
            >
              {view === "home" ? (
                <>
                  <Home className="w-5 h-5" />
                  <span className="font-bold">Home</span>
                  {folders.length > 0 && <ChevronDown className="w-4 h-4" />}
                </>
              ) : folder && view === "folder" ? (
                <>
                  <Folder className="w-5 h-5" />
                  <span className="font-bold">{extractFilename(folder)}</span>
                  <ChevronDown className="w-4 h-4" />
                </>
              ) : null}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent className="bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 p-1 min-w-[200px]">
            {view !== "home" && (
              <DropdownMenuItem
                className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 cursor-pointer rounded-md"
                onClick={() => {
                  setView("home");
                  setFolder(null);
                }}
              >
                <Home className="mr-2 h-4 w-4" />
                <span>Home</span>
              </DropdownMenuItem>
            )}
            {folders.map(
              ({ path, error }) =>
                path !== folder &&
                error == null && (
                  <DropdownMenuItem
                    key={path}
                    className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 cursor-pointer rounded-md"
                    onClick={() => {
                      setView("folder");
                      setFolder(path);
                    }}
                  >
                    <Folder className="mr-2 h-4 w-4" />
                    <span>{extractFilename(path)}</span>
                  </DropdownMenuItem>
                )
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      );
    }
  }

  function renderChannelDropdown() {
    if (view !== "folder" || !folder) return null;
    if (folderView == "users") return null;
    if (folderData == null) return <LoadingSpinnerBlack />;

    return (
      <DropdownMenu
        onOpenChange={(open: boolean) => {
          setIsChannelDropdownOpen(open);
          if (!open) {
            setIsAddingChannel(false);
            setNewChannelName("");
          }
        }}
        modal={false}
      >
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            className={sharedButtonStyles}
            style={buttonStyle}
          >
            {!channelUlid ? (
              <>
                <Globe className="w-5 h-5 mr-1" />
                <span className="font-bold">Global</span>
              </>
            ) : (
              <>
                <Hash className="w-5 h-5 mr-1" />
                <span className="font-bold">
                  {getChannelName(folderData.channels[channelUlid]!.meta)}
                </span>
              </>
            )}
            <ChevronDown className="w-4 h-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          {channelUlid && (
            <DropdownMenuItem
              className="bg-white rounded-md border-2 border-gray-100 p-1 min-w-[200px]"
              onClick={() => {
                setChannel(null);
              }}
            >
              <Globe className="w-5 h-5 mr-1" />
              <span className="font-bold">Global</span>
            </DropdownMenuItem>
          )}
          {Object.entries(folderData.channels).map(
            ([ulid, c]) =>
              ulid !== channelUlid && (
                <DropdownMenuItem
                  key={ulid}
                  className="bg-white rounded-md border-2 border-gray-100 p-1 min-w-[200px]"
                  onClick={() => {
                    setChannel(ulid);
                  }}
                >
                  <Hash className="w-5 h-5 mr-1" />
                  <span className="font-bold">{getChannelName(c!.meta)}</span>
                </DropdownMenuItem>
              )
          )}
          {isAddingChannel ? (
            <div className="px-4 py-2">
              <Input
                type="text"
                placeholder="New channel name"
                value={newChannelName}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                  setNewChannelName(e.target.value)
                }
                onKeyDown={handleAddChannel}
                autoFocus
              />
            </div>
          ) : (
            <DropdownMenuItem
              className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 cursor-pointer rounded-md"
              onClick={handleAddChannelClick}
            >
              <Plus className="mr-2 h-4 w-4" />
              <span>Create a channel</span>
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // function renderUsers() {
  //   if (view !== "folder") return null;

  //   return (
  //     <Button
  //       variant="ghost"
  //       className={`${sharedButtonStyles} absolute translate-y-full mt-2 rounded-2xl py-3 w-48`}
  //       style={buttonStyle}
  //       onClick={() => {
  //         if (folderView == "users") {
  //           setFolderView(lastFolderView);
  //         } else {
  //           setLastFolderView(lastFolderView);
  //           setFolderView("users");
  //         }
  //       }}
  //     >
  //       {folderView == "users" ? (
  //         <>
  //           <Undo2 className="w-5 h-5 mr-1" />
  //           <span className="font-bold">Back to Folder</span>
  //         </>
  //       ) : (
  //         <>
  //           <User className="w-5 h-5 mr-1" />
  //           <span className="font-bold">Users</span>
  //         </>
  //       )}
  //     </Button>
  //   );
  // }

  function renderViewToggle() {
    if (view !== "folder") return null;
    if (folderView == "users") return null;

    return (
      <div className={sharedButtonStyles} style={buttonStyle}>
        <ViewToggle />
      </div>
    );
  }

  function renderSettings() {
    if (view !== "home") return null;

    return (
      <div className={sharedButtonStyles} style={buttonStyle}>
        <Button onClick={() => setSettingsOpen(true)} className="rounded-full">
          <Settings />
        </Button>
      </div>
    );
  }

  function renderPlug() {
    if (view !== "home") return null;

    return (
      <div className={sharedButtonStyles} style={buttonStyle}>
        <Button onClick={() => setPluginsOpen(true)} className="rounded-full">
          <Plug />
        </Button>
      </div>
    );
  }

  return (
    <div className={`${containerStyles} pointer-events-none`} ref={elementRef}>
      {/* {renderUsers()} */}
      {renderFolderDropdown()}
      {renderChannelDropdown()}
      {renderViewToggle()}
      <div className="ml-auto flex flex-row">
        {renderPlug()}
        {renderSettings()}
      </div>
    </div>
  );
}
