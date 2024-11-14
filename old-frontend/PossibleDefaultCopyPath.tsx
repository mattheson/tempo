import { useStore } from "./Store";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

// dialog which asks user if they want to set a default copy path
// is this really needed?
export function PossibleDefaultCopyPath() {
  const [
    possibleDefaultCopyPath,
    setPossibleDefaultCopyPath,
    setDefaultProjectCopyPath,
    setAskUserAboutCopyPath,
  ] = useStore((state) => [
    state.possibleDefaultCopyPath,
    state.setPossibleDefaultCopyPath,
    state.setDefaultProjectCopyPath,
    state.setAskUserAboutCopyPath,
  ]);
  return (
    <AlertDialog open={possibleDefaultCopyPath != null}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Use as default copy path?</AlertDialogTitle>
        </AlertDialogHeader>
        <AlertDialogDescription>
          Would you like Tempo to save copies of projects in this directory by
          default? You can change the default copy directory in the settings.
        </AlertDialogDescription>
        <AlertDialogFooter>
          <AlertDialogAction
            onClick={() => {
              setDefaultProjectCopyPath(possibleDefaultCopyPath);
              setPossibleDefaultCopyPath(null);
              setAskUserAboutCopyPath(false);
            }}
          >
            Yes
          </AlertDialogAction>
          <AlertDialogCancel onClick={() => setPossibleDefaultCopyPath(null)}>
            No
          </AlertDialogCancel>
          <AlertDialogCancel
            onClick={() => {
              setPossibleDefaultCopyPath(null);
              setAskUserAboutCopyPath(false);
            }}
          >
            No, don't ask again
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
