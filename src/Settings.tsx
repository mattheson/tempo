import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useStore } from "./Store";
import { SettingsForm } from "./SettingsForm";

export function Settings() {
  const [settingsOpen, setSettingsOpen] = useStore((state) => [
    state.settingsOpen,
    state.setSettingsOpen,
  ]);

  return (
    <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
      <DialogContent className="min-w-[calc(100%-5em)] max-h-[calc(100%-5em)]">
        <DialogHeader>
          <DialogTitle className="text-2xl">Settings</DialogTitle>
        </DialogHeader>
        <SettingsForm />
      </DialogContent>
    </Dialog>
  );
}
