import { ChatView } from "./ChatView";
import { useStore } from "../Store";
import { UsersView } from "./UsersView";
import { TreeView } from "./TreeView";
import { PossibleDefaultCopyPath } from "@/PossibleDefaultCopyPath";

export function FolderView() {
  const [folderView, _askUserAboutCopyPath] = useStore((state) => [
    state.folderView,
    state.askUserAboutCopyPath,
  ]);

  return (
    <div className="flex flex-col min-h-screen bg-gradient-to-br from-gray-50 to-orange-50 font-sans pb-6">
      {/* this acts as padding, feels very unclean */}
      <div className="flex h-28" />
      <div className="flex flex-grow">
        {folderView == "chat" && (
          <div className="px-10 flex flex-grow">
            <ChatView />
          </div>
        )}
        {folderView == "users" && (
          <div className="px-8 flex flex-grow">
            <UsersView />
          </div>
        )}
        {folderView == "tree" && (
          <div className="flex-grow select-none">
            <TreeView />
          </div>
        )}
      </div>
      <div className="flex h-16" />
      <PossibleDefaultCopyPath />
    </div>
  );
}
