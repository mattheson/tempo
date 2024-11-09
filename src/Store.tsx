import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { StateStorage, createJSONStorage, persist } from "zustand/middleware";
import { FolderViews, Views } from "./types";
import { useEffect, useRef } from "react";
import { load, Store } from "@tauri-apps/plugin-store";
import {
  getFolderData,
  getStorePath,
  InvokePromise,
  scanFolders,
} from "./commands";
import { BackendError } from "@bindings/BackendError";
import { toast } from "sonner";
import { FolderInfo } from "@bindings/FolderInfo";
import { FolderData } from "@bindings/FolderData";
import equal from "fast-deep-equal";
import { AudioFileInfo } from "@bindings/AudioFileInfo";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/*
big TODO

all notes in channels are loaded into webview
this is a very suboptimal way of managing state for now and will not work well if folders have lots of notes 
it would be preferable to build some kind of sqlite index of notes and do stuff with that

*/

export let tauriStore: Store | null = null;

// TODO add good error handling in case any wacky stuff happens here

async function getTauriStore(): Promise<Store> {
  if (tauriStore == null) {
    const p = await getStorePath();
    tauriStore = await load(p);
    return new Promise((r) => r(tauriStore!));
  } else {
    return new Promise((r) => r(tauriStore!));
  }
}

const tauriStorage: StateStorage = {
  getItem: async (name: string): Promise<string | null> => {
    return getTauriStore().then(async (s) => {
      let val = await s.get(name);
      console.log(val);
      if (val) return val as string;
      return null;
    });
  },
  setItem: async (name: string, value: string): Promise<void> => {
    return getTauriStore().then(async (s) => {
      s.set(name, value);
    });
  },
  removeItem: async (name: string): Promise<void> => {
    return getTauriStore().then(async (s) => {
      s.delete(name);
    });
  },
};

export function useTauriFocus() {
  const unlistenBlur = useRef<UnlistenFn | null>(null);
  const unlistenFocus = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    (async () => {
      unlistenBlur.current = await listen("tauri://blur", (_) => {
        useStore.setState({ focused: false });
      });

      unlistenFocus.current = await listen("tauri://focus", (_) => {
        useStore.setState({ focused: true });
      });
    })();

    return () => {
      unlistenBlur.current?.();
      unlistenFocus.current?.();
    };
  });
}

export function useFolderInfoPoll() {
  useEffect(() => {
    useStore.getState().pollFoldersOnce();

    const id = setInterval(() => {
      useStore.getState().pollFoldersOnce();
    }, 5000);

    return () => {
      clearInterval(id);
    };
  }, []);
}

interface TempoStore {
  // persistent state (stored in tauri store) ---------------------------
  defaultUsername: string | null; // if this is null we need to do setup flow
  setDefaultUsername: (name: string) => void;

  defaultProjectCopyPath: string | null;
  setDefaultProjectCopyPath: (path: string | null) => void;

  // ask user if they want to set a path as their default copy path
  askUserAboutCopyPath: boolean;
  setAskUserAboutCopyPath: (ask: boolean) => void;

  requireRender: boolean;
  setRequireRender: (require: boolean) => void;

  _hasHydrated: boolean;
  _setHasHydrated: (h: boolean) => void;
  // --------------------------------------------------------------------

  addError: (msg: string) => void;

  // for handling any errors from invoke
  invokeWithError: (promise: InvokePromise<any>) => InvokePromise<any>;

  // whether we need setup flow for full disk access
  needMacOSSetup: boolean | null;
  setNeedMacOSSetup: (needSetup: boolean) => void;

  view: Views;
  setView: (view: Views) => void;

  folderView: FolderViews;
  setFolderView: (view: FolderViews) => void;

  settingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;

  pluginsOpen: boolean;
  setPluginsOpen: (open: boolean) => void;

  folder: string | null;
  setFolder: (folder: string | null) => void;

  channelUlid: string | null;
  setChannel: (channelUlid: string | null) => void;

  canPlayerCaptureKeys: boolean;
  setCanPlayerCaptureKeys: (b: boolean) => void;

  playingInfo: AudioFileInfo | null;
  setPlayingInfo: (audio: AudioFileInfo | null) => void;

  playing: boolean;
  setPlaying: (playing: boolean) => void;

  focused: boolean;
  setFocused: (focused: boolean) => void;

  // if non-null default copy path dialog will open
  possibleDefaultCopyPath: string | null;
  setPossibleDefaultCopyPath: (path: string | null) => void;

  // actual application data ----------------------------------------------
  folders: FolderInfo[] | null;
  setFolders: (folders: FolderInfo[]) => void;

  pollFoldersOnce: () => Promise<any>;

  folderData: FolderData | null;
  setFolderData: (data: FolderData) => void;

  pollFolderDataOnce: () => Promise<any>;

  _pollFolderData: (folder: string) => Promise<any>;

  _folderPollingId: any | null;
  // --------------------------------------------------------------------
}

export const useStore = create<TempoStore>()(
  immer(
    persist(
      (set, get) => ({
        defaultUsername: null,
        setDefaultUsername: (name: string) => {
          set((state: TempoStore) => {
            state.defaultUsername = name;
          });
        },

        defaultProjectCopyPath: null,
        setDefaultProjectCopyPath: (path: string | null) => {
          set((state: TempoStore) => {
            state.defaultProjectCopyPath = path;
          });
        },

        askUserAboutCopyPath: true,
        setAskUserAboutCopyPath: (ask: boolean) => {
          set((state: TempoStore) => {
            state.askUserAboutCopyPath = ask;
          });
        },

        requireRender: false,
        setRequireRender: (require: boolean) => {
          set((state: TempoStore) => {
            state.requireRender = require;
          });
        },

        _hasHydrated: false,
        _setHasHydrated: (h: boolean) => {
          set((state: TempoStore) => {
            state._hasHydrated = h;
          });
        },

        addError: (msg: string) => toast.error(msg),

        invokeWithError: (promise: InvokePromise<any>) => {
          console.log(promise);
          return promise.catch((err: BackendError | string) => {
            console.error(err);
            if (typeof err == "string") {
              get().addError(err);
            } else {
              const [[_, msg]] = Object.entries(err);
              get().addError(msg);
            }
          });
        },

        needMacOSSetup: null,
        setNeedMacOSSetup: (needSetup: boolean) =>
          set((state) => {
            state.needMacOSSetup = needSetup;
          }),

        view: "home",

        setView: (view: Views) =>
          set((state: TempoStore) => {
            state.view = view;
            if (view == "home") {
              state.folderData = null;
            }
          }),

        folderView: "chat",
        setFolderView: (view: FolderViews) =>
          set((state: TempoStore) => {
            state.folderView = view;
          }),

        settingsOpen: false,
        setSettingsOpen: (open: boolean) =>
          set((state: TempoStore) => {
            if (state.view != "home") return;
            state.settingsOpen = open;
          }),

        pluginsOpen: false,
        setPluginsOpen: (open: boolean) => {
          set((state: TempoStore) => {
            state.pluginsOpen = open;
          });
        },

        folder: null,
        setFolder: (folder: string | null) =>
          set((state: TempoStore) => {
            state.channelUlid = null;

            // poll for folder data
            if (state.folder != folder) {
              state.folder = folder;

              if (state._folderPollingId != null) {
                clearInterval(state._folderPollingId);
                state._folderPollingId = null;
              }
              if (folder) {
                get()._pollFolderData(folder);

                state._folderPollingId = setInterval(() => {
                  get().pollFolderDataOnce();
                }, 5000);
              }
            }

            state.folder = folder;
          }),

        channelUlid: null,
        setChannel: (channelUlid: string | null) =>
          set((state: TempoStore) => {
            state.channelUlid = channelUlid;
          }),

        canPlayerCaptureKeys: false,
        setCanPlayerCaptureKeys: (b: boolean) =>
          set((state: TempoStore) => {
            state.canPlayerCaptureKeys = b;
          }),

        playingInfo: null,

        setPlayingInfo: (info: AudioFileInfo | null) => {
          set((state: TempoStore) => {
            state.playingInfo = info;
          });
        },

        playing: false,
        setPlaying: (playing: boolean) =>
          set((state: TempoStore) => {
            state.playing = playing;
          }),

        focused: true,
        setFocused: (focused: boolean) =>
          set((state: TempoStore) => {
            state.focused = focused;
          }),

        possibleDefaultCopyPath: null,
        setPossibleDefaultCopyPath: (path: string | null) => {
          set((state: TempoStore) => {
            state.possibleDefaultCopyPath = path;
          });
        },

        folders: null,
        setFolders: (folders: FolderInfo[]) =>
          set((state: TempoStore) => {
            state.folders = folders;

            folders.forEach((i) => {
              // navigate away from folder if it became invalid
              if (i.path == state.folder && i.error != null) {
                get().addError(
                  `Folder ${i.path} became invalid, error: ${i.error}`
                );
                get().setFolder(null);
                state.view = "home";
              }
            });
          }),

        pollFoldersOnce: () => {
          if (!get().focused) return Promise.resolve(null);

          return scanFolders().then(
            (f) => {
              if (!equal(f, get().folders)) {
                get().setFolders(f);
              }
            },
            (err) => {
              console.error(`error while scanning folders: ${err}`);
            }
          );
        },

        folderData: null,
        setFolderData: (data: FolderData) =>
          set((state: TempoStore) => {
            state.folderData = data;
          }),

        pollFolderDataOnce: () => {
          if (!get().focused) return Promise.resolve(null);

          const folder = get().folder;

          if (!folder || get().view != "folder") return Promise.resolve(null);

          return get()._pollFolderData(folder);
        },

        // do not use this
        _pollFolderData: (folder: string) => {
          return getFolderData(folder).then(
            (d) => {
              if (!equal(get().folderData, d)) {
                console.log(d);
                get().setFolderData(d);
              }
            },
            (err) => {
              get().addError(
                `Error while loading folder data: ${JSON.stringify(err)}`
              );
            }
          );
        },

        _folderPollingId: null,
      }),

      {
        name: "settings",
        partialize: (state) => ({
          defaultUsername: state.defaultUsername,
          defaultProjectCopyPath: state.defaultProjectCopyPath,
          askUserAboutCopyPath: state.askUserAboutCopyPath,
          requireRender: state.requireRender,
        }),
        // TODO change this so that it's not json storage so it's more readable
        storage: createJSONStorage(() => tauriStorage),
        onRehydrateStorage: (state) => {
          return () => {
            state._setHasHydrated(true);
          };
        },
      }
    )
  )
);
