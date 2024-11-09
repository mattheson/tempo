// @ts-nocheck

import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ProjectAttachment } from "../attachment/ProjectAttachment";
import { Comments } from "./Comments";
import { useStore } from "../Store";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { ProjectInfo } from "@bindings/ProjectInfo";
import { useState } from "react";

// popup that shows detailed info about project
// along with comments
export function NoteModal({
  channelUlid,
  noteUlid,
  title,
  project
} : {
  channelUlid: string | null;
  noteUlid: string;
  title: string;
  project: ProjectInfo;
}) {
  const [open, setOpen] = useState<boolean>(false);

  return (
    <Dialog modal open={open} onOpenChange={setOpen}>
      <VisuallyHidden>
        <DialogTitle>Project {title}</DialogTitle>
      </VisuallyHidden>
      <DialogContent className="min-w-[calc(100%-5em)]">
          <>
            <ProjectAttachment
              projectPath={project.filename}
              // renderPath={project.}
              projectTitle={note.project.title}
            />
            <Comments />
          </>
      </DialogContent>
    </Dialog>
  );
}
