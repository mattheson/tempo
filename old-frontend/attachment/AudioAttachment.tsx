import { extractFilename, truncateAttachmentThing } from "../misc";
import { Music } from "lucide-react";
import { ReactNode } from "react";

export function AudioAttachment({
  filename,
  children,
}: {
  filename: string;
  children?: ReactNode;
}) {
  return (
    <div className="flex p-4 m-2">
      <div>
        <Music size="40" className="items-center justify-center h-full" />
      </div>
      <div className="flex flex-col mx-4 justify-center">
        <b className="text-nowrap">
          {truncateAttachmentThing(extractFilename(filename))}
        </b>
      </div>
      <div className="flex flex-col h-full w-full align-center">
        <div className="flex items-center h-full">{children}</div>
      </div>
    </div>
  );
}
