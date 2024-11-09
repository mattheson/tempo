import { Check, ChevronDown, Download, X } from "lucide-react";
import { useStore } from "../Store";
import { useMemo, useState } from "react";
import { LoadingSpinnerWhite } from "../misc";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { open } from "@tauri-apps/plugin-shell";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { SharedProjectData } from "@bindings/SharedProjectData";
import { PluginTable } from "@/tables/PluginTable";
import { ScrollArea } from "@/components/ui/scroll-area";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { copyProject } from "../commands";

export function CopyProjectButton({
  channelUlid,
  noteUlid,
  title,
  projectData,
}: {
  channelUlid: string | null;
  noteUlid: string;
  title: string;
  projectData: SharedProjectData;
}) {
  const [
    folder,
    invokeWithError,
    askUserAboutCopyPath,
    setPossibleDefaultCopyPath,
    defaultProjectCopyPath,
  ] = useStore((state) => [
    state.folder!,
    state.invokeWithError,
    state.askUserAboutCopyPath,
    state.setPossibleDefaultCopyPath,
    state.defaultProjectCopyPath,
  ]);

  const [copying, setCopying] = useState(false);
  const [copyPath, setCopyPath] = useState<string | null>(null);
  const [checkModalOpen, setCheckModalOpen] = useState(false);

  const missingFiles = useMemo(
    () => projectData.Ableton.missing_files.length > 0,
    [projectData]
  );

  const missingPlugins = useMemo(
    () => projectData.Ableton.missing_plugins.length > 0,
    [projectData]
  );

  const [bgColor, hover] = useMemo(
    () =>
      copyPath
        ? ["bg-gray-500", "hover:bg-gray-500/90"]
        : missingFiles || missingPlugins
        ? ["bg-red-400", "hover:bg-red-400/90"]
        : ["bg-green-400", "hover:bg-green-400/90"],
    [missingFiles, missingPlugins, copyPath]
  );

  function saveCopy() {
    setCopying(true);

    (async () => {
      console.log("copying: ", folder, channelUlid, noteUlid);
      if (defaultProjectCopyPath == null) {
        openDialog({
          multiple: false,
          directory: true,
          title: "Select a directory to copy into",
        }).then((dir) => {
          if (dir != null) {
            actuallyCopy(dir);
          } else {
            setCopying(false);
          }
        });
      } else {
        actuallyCopy(defaultProjectCopyPath);
      }
    })();
  }

  function actuallyCopy(dir: string) {
    invokeWithError(copyProject(folder, channelUlid, noteUlid, dir))
      .then(([path, errs]) => {
        console.log("copied to ", path);
        console.warn("missing files:", errs);
        setCopyPath(path);
        if (askUserAboutCopyPath && defaultProjectCopyPath == null)
          setPossibleDefaultCopyPath(dir);
      })
      .finally(() => {
        setCopying(false);
      });
  }

  function renderCopying() {
    return (
      <div className="flex items-center">
        <LoadingSpinnerWhite className="mr-2" />
        <span>Copying...</span>
      </div>
    );
  }

  function renderMissing() {
    const text =
      missingFiles && missingPlugins
        ? "Missing files and plugins"
        : missingFiles
        ? "Missing files"
        : missingPlugins && "Missing plugins";

    return (
      <div
        onClick={() => {
          setCheckModalOpen(true);
        }}
        className="flex items-center"
      >
        <X color="white" size={25} className="mr-2" />
        <span>{text}</span>
      </div>
    );
  }

  function renderOk() {
    return (
      <div onClick={saveCopy} className="flex items-center">
        <Download color="white" size={25} className="mr-2" />
        <span>Create a copy</span>
      </div>
    );
  }

  function renderCopied() {
    return (
      <div
        onClick={() => {
          open(copyPath!);
        }}
        className="flex items-center"
      >
        <Check color="white" size={25} className="mr-2" />
        <span>Click to open</span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <ChevronDown color="white" className="ml-2" />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem
              onMouseDown={(e) => {
                e.stopPropagation();
                setCopyPath(null);
                saveCopy();
              }}
            >
              Create another copy
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    );
  }

  function renderDialog() {
    const fileHelper = (
      <p>
        Your sync service might still be copying some of the referenced files.
      </p>
    );

    const pluginHelper = (
      <p>
        To avoid plugin mismatches, ensure you and your collaborators use the
        same plugins.
      </p>
    );

    const continueText = (
      <p>You can continue creating a copy of this project if you'd like.</p>
    );

    function dialogText() {
      if (missingFiles && missingPlugins) {
        return (
          <>
            <b>Referenced files and used plugins in {title} are missing.</b>
            <br />
            <br />
            {fileHelper}
            <br />
            {pluginHelper}
            {continueText}
            <p>The following files and plugins are missing:</p>
            <br />
          </>
        );
      } else if (missingFiles) {
        return (
          <>
            <b>Referenced files in {title} are missing.</b>
            <br />
            <br />
            {fileHelper}
            <br />
            {continueText}
            <p>The following files are missing:</p>
            <br />
          </>
        );
      } else {
        return (
          <>
            <b>You are missing plugins used in {title}.</b>
            <br />
            <br />
            {pluginHelper}
            <br />
            {continueText}
            <p>You are missing the following plugins:</p>
            <br />
          </>
        );
      }
    }

    return (
      <Dialog open={checkModalOpen} onOpenChange={(b) => setCheckModalOpen(b)}>
        <DialogContent className="min-w-[calc(100%-5em)]">
          <DialogHeader>
            <DialogTitle className="text-2xl">
              Missing{" "}
              {missingFiles && missingPlugins
                ? "Files and Plugins"
                : missingFiles
                ? "Files"
                : "Plugins"}{" "}
              in {title}
            </DialogTitle>
          </DialogHeader>
          <ScrollArea className="max-h-[calc(100vh-15rem)] w-full pr-5">
            {dialogText()}
            {missingFiles && missingPlugins && (
              <p className="text-2xl font-semibold pb-2">Files</p>
            )}
            {projectData.Ableton.missing_files.map((file) => (
              <div className="mb-2" key={file}>
                {file}
              </div>
            ))}
            {missingFiles && missingPlugins && (
              <p className="text-2xl font-semibold pb-2">Plugins</p>
            )}
            <PluginTable plugins={projectData.Ableton.missing_plugins} />
          </ScrollArea>
          <DialogFooter>
            <Button
              type="button"
              onClick={() => {
                setCheckModalOpen(false);
                saveCopy();
              }}
            >
              Continue Copying
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  function render() {
    return (
      <div
        className={`rounded-md text-white font-bold flex items-center p-2 cursor-pointer shadow-sm h-min ${bgColor} ${hover} ml-2`}
      >
        {copying ? (
          renderCopying()
        ) : copyPath != null ? (
          renderCopied()
        ) : missingFiles || missingPlugins ? (
          <>
            {renderMissing()}
            {renderDialog()}
          </>
        ) : (
          renderOk()
        )}
      </div>
    );
  }

  return render();
}
