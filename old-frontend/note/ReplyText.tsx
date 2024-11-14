import { useStore } from "../Store";
import { LoadingSpinnerBlack } from "../misc";
import { SharedNote } from "@bindings/SharedNote";

export function ReplyText({
  channelUlid,
  noteUlid,
}: {
  channelUlid: string | null;
  noteUlid: string;
}) {
  const [folderData] = useStore((state) => [state.folderData]);

  function render() {
    if (folderData == null) return <LoadingSpinnerBlack />;
    const note = channelUlid
      ? folderData.channels[channelUlid]?.notes[noteUlid]
      : folderData.global[noteUlid];

    if (note) {
      const [[res, maybeNote]] = Object.entries(note);
      if (res == "Ok") {
        const note = maybeNote as SharedNote;
        return (
          <p>
            Replying to
            <>
              <b> {note.sender}:</b>{" "}
              {note.body.length != 0 ? (
                note.body.length > 30 ? (
                  note.body.slice(0, 30) + "..."
                ) : (
                  note.body
                )
              ) : (
                <i>No note body</i>
              )}
            </>
          </p>
        );
      } else {
        const err = maybeNote as string;
        return <i className="text-red-300">Error while loading note: {err}</i>;
      }
    } else {
      <i className="text-red-300">Unknown note</i>;
    }
  }

  return render();
}
