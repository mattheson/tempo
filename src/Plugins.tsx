import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useStore } from "./Store";
import { useEffect, useState } from "react";
import { LoadingSpinnerBlack } from "./misc";
import { getLastPluginScanTime, scanPlugins } from "./commands";
import { Button } from "@/components/ui/button";

export function Plugins() {
  const [pluginsOpen, setPluginsOpen, invokeWithError] = useStore((state) => [
    state.pluginsOpen,
    state.setPluginsOpen,
    state.invokeWithError,
  ]);

  const [lastScanTime, setLastScanTime] = useState<Date | null | string>(null);
  const [scanning, setScanning] = useState<null | string | boolean>(false);

  useEffect(() => {
    if (!pluginsOpen || scanning == true) return;

    (async () => {
      invokeWithError(getLastPluginScanTime()).then(
        (time) => {
          setLastScanTime(new Date(time * 1000));
        },
        (err) => {
          console.error(`error while scanning plugins: ${err}`);
          setLastScanTime(err);
        }
      );
    })();
  }, [pluginsOpen, scanning]);

  function scan() {
    setScanning(true);
    invokeWithError(
      scanPlugins().then(
        () => {
          setScanning(false);
        },
        (err) => {
          setScanning(err);
        }
      )
    );
  }

  return (
    <Dialog open={pluginsOpen} onOpenChange={setPluginsOpen}>
      <DialogContent className="min-w-[calc(100%-5em)] max-h-[calc(100%-5em)]">
        <DialogHeader>
          <DialogTitle className="text-2xl">Plugins</DialogTitle>
        </DialogHeader>
        Tempo maintains its own plugin database which scans Ableton's plugin database.
        <br />
        If you've installed new plugins, ensure you've scanned them in Ableton
        beforehand.
        <br />
        This database is copied into folders so that other users can see your installed plugins.
        <br />
        Pressing the button below will rescan your plugins and copy the new database into all valid folders.
        {lastScanTime instanceof Date ? (
          <p>
            <b>Last scan time: </b>
            {lastScanTime.toLocaleString("en-US", {
              year: "numeric",
              month: "short",
              day: "numeric",
              hour: "2-digit",
              minute: "2-digit",
            })}
          </p>
        ) : lastScanTime == null ? (
          <LoadingSpinnerBlack />
        ) : (
          <p className="text-red-500">{lastScanTime}</p>
        )}
        <Button onClick={scan} disabled={scanning == null}>
          {scanning == true ? <LoadingSpinnerBlack /> : <>Scan plugins</>}
        </Button>
        {typeof scanning == "string" && (
          <p className="text-red-500">Error: {scanning}</p>
        )}
      </DialogContent>
    </Dialog>
  );
}
