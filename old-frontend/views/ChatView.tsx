import { useEffect, useRef, useState } from "react";
import { Note } from "../note/Note";
import { NoteBox } from "../note/NoteBox";
import { useStore } from "../Store";
import useResizeObserver from "@react-hook/resize-observer";
import { LoadingSpinnerBlack } from "../misc";
import { NewAttachment } from "@bindings/NewAttachment";

// TODO support drag/drop of ableton project folder (and other attachments) rather than just .als only
// TODO remove hard coded transform offsets

export function ChatView() {
  const [channelUlid, folderData] = useStore((state) => [
    state.channelUlid,
    state.folderData,
  ]);

  const [replyUlid, setReplyUlid] = useState<string | null>(null);
  const [attachment, setAttachment] = useState<NewAttachment | null>(null);
  const [noteBoxHeight, setNoteBoxHeight] = useState(0);

  const noteBoxRef = useRef<HTMLDivElement | null>(null);
  const notesEndRef = useRef<HTMLDivElement | null>(null);

  useResizeObserver(noteBoxRef, (e) => {
    setNoteBoxHeight(e.contentRect.height);
  });

  useEffect(() => {
    if (noteBoxRef.current) {
      setNoteBoxHeight(noteBoxRef.current.clientHeight);
    }
  }, [noteBoxRef]);

  // useLayoutEffect(() => {
  //   notesEndRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
  // }, [folderData]);

  function render() {
    if (folderData == null) return <LoadingSpinnerBlack />;
    return renderNotes();
  }

  function renderNotes() {
    const notes = channelUlid
      ? folderData!.channels[channelUlid]!.notes
      : folderData!.global;

    return (
      <div className="flex-grow overflow-y-auto overflow-x-hidden">
        <div className="text-pretty max-w-full break-words">
          {Object.entries(notes)
            .sort()
            .map(([ulid, note]) => (
              <div className="my-2" key={ulid}>
                <Note
                  channelUlid={channelUlid}
                  note={note!}
                  noteUlid={ulid}
                  onReply={() => {
                    setReplyUlid(ulid);
                  }}
                  noteBottom="comments"
                />
              </div>
            ))}
          <div ref={notesEndRef} />
        </div>
      </div>
    );
  }

  return (
    <div
      className="flex flex-col w-full h-full transition-all duration-300 ease-in-out"
      style={{
        paddingBottom: `${(noteBoxHeight + 50).toString() + "px"}`,
      }}
    >
      {render()}
      <div className="fixed bottom-24 left-4 right-4 z-10 transition-all duration-300 ease-in-out">
        <NoteBox
          channelUlid={channelUlid}

          attachment={attachment}
          setAttachment={setAttachment}

          replyUlid={replyUlid}
          setReplyUlid={setReplyUlid}

          ref={noteBoxRef}
        />
      </div>
    </div>
  );
}
