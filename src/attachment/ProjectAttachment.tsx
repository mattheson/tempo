import { FileMusic } from "lucide-react";
import { ReactNode } from "react";
import {
  extractFilename,
  truncateAttachmentThing,
} from "../misc";

// base for project attachments

export function ProjectAttachment({
  projectPath,

  projectType,

  children,
}: {
  projectPath: string;

  projectType: string;

  children?: ReactNode;
}) {
  // const addError = useStore((state) => state.addError);

  return (
    <div className="flex p-4 m-2">
      <div>
        <FileMusic size="40" className="items-center justify-center h-full" />
      </div>
      <div className="flex flex-col mx-4 justify-center">
        <b className="text-nowrap">{truncateAttachmentThing(extractFilename(projectPath))}</b>
        <i className="text-nowrap">{projectType}</i>
      </div>
      <div className="flex flex-col h-full w-full align-center">
        <div className="flex items-center h-full">{children}</div>
      </div>
    </div>
  );
}
