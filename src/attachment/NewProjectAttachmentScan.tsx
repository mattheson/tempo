import { useEffect, useState } from "react";
import { extractFilename, LoadingSpinnerBlack } from "../misc";
import { useStore } from "../Store";
import { CheckCircle, CircleAlert, RefreshCcw } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { scanProjectFileRefs, scanProjectPlugins } from "../commands";
import { ProjectFileRefScan } from "@bindings/ProjectFileRefScan";
import { PluginScan } from "@bindings/PluginScan";
import { PluginTable } from "@/tables/PluginTable";
import { ScrollArea } from "@/components/ui/scroll-area";

export function NewProjectAttachmentScan({
  projectPath,
  setScanningProject,
}: {
  projectPath: string;
  setScanningProject: (b: boolean) => void;
}) {
  const [invokeWithError, folder, folderData] = useStore((state) => [
    state.invokeWithError,
    state.folder!,
    state.folderData!,
  ]);

  const [fileRefScan, setFileRefScan] = useState<ProjectFileRefScan | null>(
    null
  );
  const [pluginScan, setPluginScan] = useState<PluginScan | null>(null);

  const [fileScanErr, setFileScanErr] = useState<string | null>(null);
  const [pluginScanErr, setPluginScanErr] = useState<string | null>(null);

  const [dialogContent, setDialogContent] = useState<"files" | "plugins">(
    "files"
  );

  const [dialogOpen, setDialogOpen] = useState<boolean>(false);

  function scan() {
    setFileRefScan(null);
    setPluginScan(null);

    setScanningProject(true);

    setFileScanErr(null);
    setPluginScanErr(null);

    (async () => {
      const files = invokeWithError(scanProjectFileRefs(projectPath)).then(
        (scan) => {
          setFileRefScan(scan);
          console.log(scan);
        },
        (err) => setFileScanErr(err)
      );

      const plugins = invokeWithError(
        scanProjectPlugins(folder, projectPath)
      ).then(
        (scan) => {
          setPluginScan(scan);
          console.log("plugins:", scan);
        },
        (err) => setPluginScanErr(err)
      );

      await files;
      await plugins;
    })().finally(() => {
      setScanningProject(false);
    });
  }

  useEffect(() => {
    scan();
  }, [projectPath]);

  function renderFiles() {
    if (fileRefScan == null && fileScanErr == null) {
      return (
        <div>
          <LoadingSpinnerBlack />
          <b className="text-xl">Scanning files...</b>
        </div>
      );
    } else if (fileScanErr) {
      return (
        <div>
          <p className="text-red-400">Error: {fileScanErr}</p>
        </div>
      );
    } else if (fileRefScan) {
      return (
        <div
          className={`rounded-md p-3 font-bold select-none cursor-pointer text-lg text-white ${
            fileRefScan.missing.length > 0 ? "bg-red-400" : "bg-green-400"
          }`}
          onClick={() => {
            setDialogContent("files");
            setDialogOpen(true);
          }}
        >
          {fileRefScan.missing.length > 0 ? (
            <>
              <CircleAlert color="white" size={25} />
              Missing files
            </>
          ) : (
            <>
              <CheckCircle color="white" size={30} />
              Found all referenced files
            </>
          )}
        </div>
      );
    }
  }

  function renderPlugins() {
    if (pluginScan == null && pluginScanErr == null) {
      return (
        <div>
          <LoadingSpinnerBlack />
          <b className="text-xl">Scanning plugins...</b>
        </div>
      );
    } else if (pluginScanErr) {
      return (
        <div>
          <p className="text-red-400">Error: {pluginScanErr}</p>
        </div>
      );
    } else if (pluginScan) {
      const missing = Object.values(pluginScan.missing).some(
        (arr) => arr!.length > 0
      );
      return (
        <div
          className={`rounded-md p-3 font-bold select-none cursor-pointer text-lg text-white ${
            missing ? "bg-red-400" : "bg-green-400"
          }`}
          onClick={() => {
            setDialogContent("plugins");
            setDialogOpen(true);
          }}
        >
          {missing ? (
            <>
              <CircleAlert color="white" size={25} />
              Missing plugins
            </>
          ) : (
            <>
              <CheckCircle color="white" size={30} />
              Plugins compatible
            </>
          )}
        </div>
      );
    }
  }

  function renderRefresh() {
    if ((fileRefScan || fileScanErr) && (pluginScan || pluginScanErr))
      return (
        <RefreshCcw
          onClick={() => {
            scan();
          }}
          className="cursor-pointer"
        />
      );
  }

  function renderFileAccord() {
    if (fileRefScan == null) return;
    if (dialogContent != "files") return;

    return fileRefScan.ok.length > 0 || fileRefScan.missing.length > 0 ? (
      <>
        {fileRefScan.ok.length > 0 && (
          <AccordionItem value="ok">
            <AccordionTrigger className="text-xl">Files found</AccordionTrigger>
            <AccordionContent>
              {fileRefScan.ok.map((ref, idx) => {
                return (
                  <div className="mb-2" key={"ok-" + idx}>
                    {ref.abs}
                  </div>
                );
              })}
            </AccordionContent>
          </AccordionItem>
        )}
        {fileRefScan.missing.length > 0 && (
          <AccordionItem value="missing">
            <AccordionTrigger className="text-xl">
              Files missing
            </AccordionTrigger>
            <AccordionContent>
              {fileRefScan.missing.map((ref, idx) => {
                return (
                  <div className="mb-2" key={"missing-" + idx}>
                    <p>{ref.file.abs}</p>
                    <p className="text-red-400">{ref.err}</p>
                    <br />
                  </div>
                );
              })}
            </AccordionContent>
          </AccordionItem>
        )}
      </>
    ) : (
      <>No files were referenced in this project.</>
    );
  }

  function renderPluginAccord() {
    if (pluginScan == null) return;
    if (dialogContent != "plugins") return;

    return pluginScan.plugins.length > 0 ? (
      <>
        <AccordionItem value="plugins">
          <AccordionTrigger className="text-xl">Plugins found</AccordionTrigger>
          <AccordionContent>
            <PluginTable plugins={pluginScan.plugins} />
          </AccordionContent>
        </AccordionItem>
        {Object.entries(pluginScan.missing).map(
          ([username, idxs]) =>
            idxs!.length > 0 && (
              <AccordionItem value={"missing-" + username} key={username}>
                <AccordionTrigger className="text-xl">
                  {username == folderData.username
                    ? `Your missing plugins`
                    : `Plugins missing for ${username}`}
                </AccordionTrigger>
                <AccordionContent>
                  <p className="pb-3">
                    {username == folderData.username ? (
                      <>
                        You are missing the following plugins. If you'd like,
                        you can continue sending this project.
                      </>
                    ) : (
                      <>
                        <b>{username}</b> does not have the following plugins
                        installed. If they've installed these plugins, tell them
                        to rescan their plugins.
                      </>
                    )}
                  </p>
                  <PluginTable
                    plugins={pluginScan.plugins.filter((p, i) =>
                      idxs?.includes(i) ? p : null
                    )}
                  />
                </AccordionContent>
              </AccordionItem>
            )
        )}
      </>
    ) : (
      <>No plugins were found in this project.</>
    );
  }

  function renderDialog() {
    return (
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="min-w-[calc(100%-5em)] max-h-[calc(100%-5em)]">
          <DialogHeader>
            <DialogTitle className="text-2xl">
              {dialogContent == "files"
                ? `Files referenced in ${extractFilename(projectPath)}`
                : `Plugins used in ${extractFilename(projectPath)}`}
            </DialogTitle>
          </DialogHeader>
          <ScrollArea className="max-h-[calc(100vh-15rem)] w-full pr-5">
            <Accordion type="multiple">
              {renderFileAccord()}
              {renderPluginAccord()}
            </Accordion>
          </ScrollArea>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <div className="flex space-x-4 items-center align-middle">
      {renderRefresh()}
      {renderFiles()}
      {renderPlugins()}
      {renderDialog()}
    </div>
  );
}
