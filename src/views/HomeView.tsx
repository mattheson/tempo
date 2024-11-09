import { Ellipsis, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { open } from "@tauri-apps/plugin-dialog";
import {
  checkFolderInsideFolder,
  createOrAddFolder,
  removeFolder,
} from "../commands";
import { useStore } from "../Store";
import { useMemo, useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { LoadingSpinnerBlack } from "../misc";
import { Settings } from "../Settings";
import { Plugins } from "@/Plugins";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { FolderInfo } from "@bindings/FolderInfo";

export function HomeView() {
  const [folders, invokeWithError, defaultUsername] = useStore((state) => [
    state.folders,
    state.invokeWithError,
    state.defaultUsername!,
  ]);

  const [loadingFolder, setLoadingFolder] = useState(false);
  const [insideFolder, setInsideFolder] = useState<{
    folder: string;
    inside: string;
  } | null>(null);
  const [insideFolderDialogOpen, setInsideFolderDialogOpen] = useState(false);

  async function setupFolder() {
    const folder = await open({
      multiple: false,
      directory: true,
      title:
        "Choose an existing Tempo folder, or choose a new folder for Tempo to initialize",
    });
    setLoadingFolder(true);
    if (folder) {
      // check whether folder is inside existing tempo folder
      invokeWithError(
        checkFolderInsideFolder(folder).then((inside) => {
          console.log("result", inside);
          if (inside == null) {
            invokeWithError(
              createOrAddFolder(folder, defaultUsername).finally(() => {
                setLoadingFolder(false);
              })
            );
          } else {
            setInsideFolder({ folder, inside });
            setInsideFolderDialogOpen(true);
          }
        })
      );
    } else {
      setLoadingFolder(false);
    }
  }

  return (
    <>
      <div className="flex flex-col min-h-full">
        <main className="flex-grow pt-20 px-8 pb-20 overflow-y-auto">
          <h1 className="text-4xl font-bold mb-4">Home</h1>
          <div className="flex items-center mb-4">
            <h2 className="text-2xl mr-2">Folders</h2>
            <Button
              size="sm"
              variant="ghost"
              className="p-1"
              onClick={() => {
                setupFolder();
              }}
            >
              <Plus className="h-5 w-5" />
            </Button>
          </div>
          <div className="flex flex-col gap-4 mb-6 flex-wrap">
            {folders == null ? (
              <LoadingSpinnerBlack />
            ) : (
              folders.map((folder) => (
                <Folder folder={folder} key={folder.path} />
              ))
            )}
            {loadingFolder && (
              <div className="bg-white p-3 rounded-lg shadow-md aspect-square flex flex-col justify-between cursor-pointer content-center">
                <div>
                  <LoadingSpinnerBlack />
                </div>
                <div className="flex -space-x-1 overflow-hidden mt-2"></div>
              </div>
            )}
          </div>
        </main>
      </div>
      <AlertDialog
        open={insideFolderDialogOpen}
        onOpenChange={setInsideFolderDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="text-2xl">
              Continue with outer folder?
            </AlertDialogTitle>
          </AlertDialogHeader>
          <div>
            The directory you provided (<b>{insideFolder?.folder}</b>) is inside
            of an existing Tempo folder.
          </div>
          <div>
            Would you like to continue with adding the outer existing folder (
            <b>{insideFolder?.inside}</b>)?
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (insideFolder) {
                  setInsideFolderDialogOpen(false);
                  invokeWithError(
                    createOrAddFolder(
                      insideFolder.inside,
                      defaultUsername
                    ).finally(() => {
                      setLoadingFolder(false);
                    })
                  );
                }
              }}
            >
              Continue
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Settings />
      <Plugins />
    </>
  );
}

function Folder({ folder }: { folder: FolderInfo }) {
  const [invokeWithError, setView, setFolder] = useStore((state) => [
    state.invokeWithError,
    state.setView,
    state.setFolder,
  ]);

  const [dropdownOpen, setDropdownOpen] = useState<boolean>(false);

  const [path, err] = useMemo(() => {
    return [folder.path, folder.error];
  }, [folder]);

  return (
    <div
      key={path}
      className="bg-white p-3 rounded-lg shadow-md flex flex-col justify-between cursor-pointer"
      onClick={() => {
        if (!err) {
          setView("folder");
          setFolder(path);
        }
      }}
    >
      <div>
        <div className="flex flex-row">
          <h3 className="text-sm font-semibold mb-1 text-gray-800">{path}</h3>
          <DropdownMenu open={dropdownOpen} onOpenChange={setDropdownOpen}>
            <DropdownMenuTrigger asChild>
              <Ellipsis className="ml-auto cursor-pointer" />
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem
                onClick={(e) => {
                  invokeWithError(removeFolder(path));
                  e.stopPropagation();
                }}
              >
                Remove folder (just removes it from Tempo, doesn't delete
                folder!)
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
        {err && <p className="text-red-400">{err}</p>}
      </div>
    </div>
  );
}

// {folders == null ? (
//   <LoadingSpinnerBlack />
// ) : (
//   folders.map(({ path, error }) => (
//     <div
//       key={path}
//       className="bg-white p-3 rounded-lg shadow-md flex flex-col justify-between cursor-pointer"
//       onClick={() => {
//         if (!error) {
//           setView("folder");
//           setFolder(path);
//         }
//       }}
//     >
//       <div>
//         <div className="flex flex-row">
//         <h3 className="text-sm font-semibold mb-1 text-gray-800">
//           {path}
//         </h3>
//         <Ellipsis className="ml-auto" />
//         </div>
//         {error && <p className="text-red-400">{error}</p>}
//       </div>
//     </div>
//   ))
// )}
