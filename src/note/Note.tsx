import { Reply } from "lucide-react";
import { useMemo, useState } from "react";
import { decodeTime } from "ulidx";
import { ReplyText } from "./ReplyText";
import { SharedNote } from "@bindings/SharedNote";
import { TempoResult } from "@bindings/TempoResult";
import { NoteAttachment } from "@/attachment/NoteAttachment";
import { Comments } from "./Comments";

export type NoteBottom = "comments" | "jump";

// a note in chat view
export function Note({
  channelUlid,
  noteUlid,

  note,

  onReply,

  noteBottom,
}: {
  channelUlid: string | null;
  noteUlid: string;

  note: TempoResult<SharedNote>;

  onReply?: () => void;

  noteBottom: NoteBottom;
}) {
  const [replyMousedOver, setReplyMousedOver] = useState<boolean>(false);

  const [ok, noteOrErr]: [boolean, SharedNote | string] = useMemo(() => {
    if ("Ok" in note) return [true, note.Ok];
    return [false, note.Err];
  }, [note]);

  function render() {
    if (ok) {
      return renderOk(noteOrErr as SharedNote);
    } else {
      return renderErr(noteOrErr as string);
    }
  }

  function renderErr(err: string) {
    return (
      <div>
        <p>
          <b>
            Error while loading note, your sync service might be syncing still:
          </b>{" "}
          {err}
        </p>
      </div>
    );
  }

  function renderOk(note: SharedNote) {
    return (
      <>
        <div className="flex justify-between items-center p-3">
          <div>
            <p className="text-wrap select-none mr-3">
              <b className="mr-2">{note.sender}</b>
              {new Date(decodeTime(noteUlid)).toLocaleString("en-US", {
                year: "numeric",
                month: "short",
                day: "numeric",
                hour: "2-digit",
                minute: "2-digit",
              })}
            </p>
            {note.reply_ulid && (
              <div className="ml-auto mr-4 flex items-center">
                <Reply className="mr-1" />
                <ReplyText
                  channelUlid={channelUlid}
                  noteUlid={note.reply_ulid}
                />
              </div>
            )}
          </div>

          {onReply && (
            <div
              className="ml-auto mr-2"
              onMouseEnter={() => setReplyMousedOver(true)}
              onMouseLeave={() => setReplyMousedOver(false)}
            >
              <Reply
                color={replyMousedOver ? "#4B5563" : "#D1D5DB"}
                size={25}
                className="cursor-pointer"
                onMouseDown={(e) => {
                  e.preventDefault();
                  onReply();
                }}
              />
            </div>
          )}
        </div>
        {note.body.length > 0 && <p className="p-3">{note.body}</p>}
        {note.attachment && (
          <NoteAttachment
            channelUlid={channelUlid}
            noteUlid={noteUlid}
            attachment={note.attachment}
          />
        )}
        {note.attachment && noteBottom == "comments" && (
          <Comments
            channelUlid={channelUlid}
            noteUlid={noteUlid}
            comments={note.comments}
          />
        )}
      </>
    );
  }

  return (
    <div className="flex flex-col relative transition-all duration-300 ease-in-out shadow-md rounded-lg bg-white border border-gray-200 w-full h-full">
      {render()}
    </div>
  );
}
