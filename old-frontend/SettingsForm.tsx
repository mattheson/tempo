import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useStore } from "./Store";
import { Checkbox } from "@/components/ui/checkbox";
import { Folder, X } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

export function SettingsForm<T>({
  doOnce,
}: {
  doOnce?: { cb: () => Promise<T>; loadingMsg: string };
}) {
  const [
    setSettingsOpen,
    setDefaultUsername,
    setRequireRender,
    setDefaultProjectCopyPath,
  ] = useStore((state) => [
    state.setSettingsOpen,
    state.setDefaultUsername,
    state.setRequireRender,
    state.setDefaultProjectCopyPath,
  ]);

  const [username, setUsername] = useState<string>("");
  const [render, setRender] = useState<boolean>(false);
  const [defaultCopy, setDefaultCopy] = useState<string | null>(null);

  const [usernameError, setUsernameError] = useState<string | null>(null);

  const [doOnceStatus, setDoOnceStatus] = useState<
    null | { doing: string } | { err: string } | "done"
  >(null);

  useEffect(() => {
    const u = useStore.getState().defaultUsername;
    const dc = useStore.getState().defaultProjectCopyPath;

    if (u != null) {
      setUsername(u);
    }

    setRender(useStore.getState().requireRender);
    setDefaultCopy(dc);
  }, []);

  function validateUsername(value: string) {
    if (value.length == 0) {
      setUsernameError("Username is required");
      return false;
    }
    if (value.length > 16) {
      setUsernameError("Username must be 16 characters or less");
      return false;
    }
    if (!/^[\x00-\x7F]*$/.test(value)) {
      setUsernameError("Username must contain only ASCII characters");
      return false;
    }
    setUsernameError(null);
    return true;
  }

  function trySetSettings() {
    if (doOnce) {
      if (doOnceStatus == null) {
        setDoOnceStatus({ doing: doOnce.loadingMsg });
        doOnce.cb().then(
          () => {
            setDoOnceStatus("done");
            setSettings();
          },
          (err) => {
            setDoOnceStatus({ err });
          }
        );
      }
    } else {
      setSettings();
    }
  }

  function setSettings() {
    if (usernameError) return;
    setDefaultUsername(username);
    setRequireRender(render);
    setDefaultProjectCopyPath(defaultCopy);

    setSettingsOpen(false);
  }

  return (
    <>
      <div className="grid gap-6 py-4">
        <div className="grid grid-cols-7 items-center gap-4">
          <Label htmlFor="username" className="text-right col-span-3">
            Default username to use in added folders
          </Label>
          <div className="col-span-4">
            <Input
              id="username"
              value={username}
              onChange={(e) => {
                setUsername(e.target.value);
                validateUsername(e.target.value);
              }}
              className={`w-full ${usernameError ? "border-red-500" : ""}`}
            />
            {usernameError && (
              <p className="text-red-500 text-sm mt-1">{usernameError}</p>
            )}
          </div>

          <Label htmlFor="requireRender" className="text-right col-span-3">
            Require render on project attachments
          </Label>
          <div className="col-span-4">
            <Checkbox
              checked={render}
              onClick={() => setRender(!render)}
              id="requireRender"
            />
          </div>

          <Label htmlFor="defaultCopyDir" className="text-right col-span-3">
            Default project copy directory (optional)
          </Label>
          <div className="col-span-4">
            <div className="flex flex-row px-3 py-1 border border-gray-200 rounded-md text-sm items-center">
              <p className="select-none cursor-default">
                {defaultCopy != null ? (
                  defaultCopy
                ) : (
                  <i>No default directory selected</i>
                )}
              </p>
              <Button
                variant="secondary"
                className="ml-auto"
                onClick={() => {
                  if (defaultCopy != null) {
                    setDefaultCopy(null);
                  } else {
                    open({
                      directory: true,
                      multiple: false,
                      title:
                        "Select a default directoy for Tempo to copy projects into",
                    }).then((path) => {
                      if (path != null) {
                        setDefaultCopy(path);
                      }
                    });
                  }
                }}
              >
                {defaultCopy != null ? <X size={20} /> : <Folder size={20} />}
              </Button>
            </div>
          </div>
        </div>
      </div>
      <Button
        type="submit"
        disabled={username == "" || usernameError != null}
        onClick={trySetSettings}
      >
        Save changes
      </Button>
      {doOnceStatus != null && doOnceStatus != "done" && (
        <div className="pt-2">
          {"err" in doOnceStatus && (
            <p className="text-red-500">Error: {doOnceStatus.err}</p>
          )}
          {"doing" in doOnceStatus && <p>{doOnceStatus.doing}</p>}
        </div>
      )}
    </>
  );
}
