import "./App.css";
import { useEffect } from "react";
import { ThemeProvider } from "@/components/theme-provider";
import { HomeView } from "./views/HomeView";
import { Setup } from "./Setup";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TopBar } from "./bar/TopBar";
import { FolderView } from "./views/FolderView";
import {
  tauriStore,
  useFolderInfoPoll,
  useStore,
  useTauriFocus,
} from "./Store";
import { fallbackRender, LoadingSpinnerBlack } from "./misc";
import { MacOSSetup } from "./MacOSSetup";
import { ErrorBoundary } from "react-error-boundary";
import { Toaster } from "sonner";
import { Player } from "./Player";
import { fatal, needFullDisk, verifyUserHasAbleton } from "./commands";

export function App() {
  const [
    needMacOSSetup,
    setNeedMacOSSetup,
    defaultUsername,
    view,
    _hasHydrated,
  ] = useStore((state) => [
    state.needMacOSSetup,
    state.setNeedMacOSSetup,
    state.defaultUsername,
    state.view,
    state._hasHydrated,
  ]);

  useTauriFocus();
  useFolderInfoPoll();

  useEffect(() => {
    verifyUserHasAbleton().then(
      (v) => {
        if (!v)
          fatal(
            'You appear to not have Ableton installed!\nTempo only supports Ableton as of now.\nTempo could not find Ableton\'s "Live Database" directory.'
          );
      },
      (err) => {
        fatal(
          "Tempo encountered an error while checking if you have Ableton installed. Ensure Tempo has Full Disk Access on macOS. Error: " +
            JSON.stringify(err)
        );
      }
    );

    // Window is initially hidden. We show it after a little bit to avoid white screen when starting up.
    setTimeout(() => {
      getCurrentWindow().show();
    }, 100);

    needFullDisk().then(
      (v) => {
        setNeedMacOSSetup(v);
      },
      (err) => {
        fatal("Error while checking if Tempo has Full Disk Access: " + err);
      }
    );
  }, []);

  function render() {
    if (tauriStore == null || !_hasHydrated) return renderLoading();

    if (needMacOSSetup != null) {
      if (needMacOSSetup) return <MacOSSetup />;
      if (defaultUsername == null) return <Setup />;

      return renderOk();
    } else {
      return renderLoading();
    }
  }

  function renderLoading() {
    return (
      <div className="bg-inherit h-screen flex items-center justify-center">
        <LoadingSpinnerBlack className="flex w-[50px] h-[50px]" />
      </div>
    );
  }

  function renderOk() {
    return (
      <>
        <TopBar />
        {view == "home" && <HomeView />}
        {view == "folder" && <FolderView />}
        <Toaster position="bottom-right" richColors closeButton />
        <div className="fixed bottom-0 w-full z-[5]">
          <Player />
        </div>
      </>
    );
  }

  return (
    <ThemeProvider defaultTheme="light" storageKey="vite-ui-theme">
      <div className="h-screen bg-gradient-to-br from-gray-50 to-orange-50 font-sans antialiased select-none">
        <ErrorBoundary fallbackRender={fallbackRender}>
          {render()}
        </ErrorBoundary>
      </div>
    </ThemeProvider>
  );
}
