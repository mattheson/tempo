//@ts-nocheck
// old browser view, file explorer-like view for stuff in a channel

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useStore } from "./Store";

// TODO maybe add some kind of top level for browser where each channel looks like a folder?

export function BrowserView() {
  const [folders, folder, channel, openProjectModal] = useStore(
    (state) => [
      state.folders,
      state.folder!,
      state.channelUlid,
      state.openProjectModal,
    ]
  );

  const channelName = channel
    ? folders[folder]?.channels[channel].meta.name
    : "Global";

  const notes = channel
    ? folders[folder]?.channels[channel]?.notes
    : folders[folder]?.global;

  // TODO this kind of stuff would be nice to do on backend in advance using sqlite db
  const withProject = Object.entries(notes).filter(
    ([_ulid, doc]) => doc.project != null
  );

  return (
    <>
      <h1 className="text-3xl font-bold p-5">Projects Added to {channelName}</h1>
      <Table className="select-none">
        <TableHeader className="cursor-default">
          <TableRow>
            <TableHead>Creator</TableHead>
            <TableHead>Title</TableHead>
            <TableHead>Filename</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody className="bg-white cursor-pointer">
          {withProject.map(([ulid, doc]) => (
            <TableRow
              key={ulid}
              onClick={() => {
                openProjectModal(ulid);
              }}
            >
              <TableCell>{doc.sender}</TableCell>
              <TableCell>{doc.project!.title}</TableCell>
              <TableCell>{doc.project!.project_hash}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </>
  );
}

// flat file view of projects
// how will global channel work?
