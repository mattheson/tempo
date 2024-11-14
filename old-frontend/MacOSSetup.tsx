import { HardDrive } from "lucide-react";
import { openFullDisk, restart } from "./commands";
import { useStore } from "./Store";
import { useState } from "react";
import { Card } from "./components/ui/card";
import { Button } from "./components/ui/button";

export function MacOSSetup() {
  const invokeWithError = useStore((state) => state.invokeWithError);
  const [openedSettings, setOpenedSettings] = useState(false);

  return (
    <div className="p-6 bg-inherit">
      <span className="flex items-center mb-5">
        <HardDrive className="mr-5 w-[100px] h-[100px]" color="gray" />{" "}
        <h1 className="text-4xl mb-5 font-bold">Full Disk Access</h1>
      </span>
      <b>Hello macOS user!</b>
      <br />
      <br />
      <b>Tempo needs Full Disk Access permissions in order to work properly.</b>
      <br />
      <br />
      <p>
        These permissions are used to scan your installed plugins and simplify
        copying files.
      </p>
      <br />
      <p>Tempo makes no direct connections to the Internet whatsoever.</p>
      <p>
        Tempo is open-source software; you can audit its source code.
      </p>
      <br />
      <br />
      <br />
      <p>Press the button below to open the Full Disk settings tab.</p>
      <b>Tempo must be restarted after the permission is granted.</b>
      <Card className="p-5 mt-5">
        <Button
          className="mr-5"
          onClick={() => {
            invokeWithError(openFullDisk());
            setTimeout(() => {
              setOpenedSettings(true);
            }, 3000);
          }}
        >
          Open Full Disk Access Settings
        </Button>
        <Button disabled={!openedSettings} onClick={restart}>
          Restart Tempo
        </Button>
      </Card>
    </div>
  );
}
