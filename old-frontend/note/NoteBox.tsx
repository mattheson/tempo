import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import TextareaAutosize from "react-textarea-autosize";
import { Button } from "@/components/ui/button";
import { Paperclip, Plus, Send, X } from "lucide-react";
import { checkWithin, LoadingSpinnerBlack, useHandleTauriDrag } from "../misc";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "../Store";
import { ReplyText } from "./ReplyText";
import { NewNote } from "@bindings/NewNote";
import { createNote, getAttachmentType } from "../commands";
import { NewAttachment } from "@bindings/NewAttachment";
import { isValidNewAttachment } from "../types";
import { NewNoteAttachment } from "../attachment/NewNoteAttachment";

export interface NoteBoxProps {
  channelUlid: string | null;

  attachment: NewAttachment | null;
  setAttachment: (attachment: NewAttachment | null) => void;

  replyUlid: string | null;
  setReplyUlid?: (replyUlid: string | null) => void;

  onSent?: () => void;
}

// textarea for writing new notes
export const NoteBox = forwardRef<HTMLDivElement, NoteBoxProps>(
  (props, ref) => {
    const [invokeWithError, requireRender] = useStore((state) => [
      state.invokeWithError,
      state.requireRender,
    ]);

    useImperativeHandle(ref, () => parentRef.current!, []);

    const parentRef = useRef<HTMLDivElement | null>(null);
    const textareaRef = useRef<HTMLTextAreaElement | null>(null);
    const dragElems = useRef<HTMLElement[]>([]);

    const [canSend, setCanSend] = useState(false);
    const [dragHover, setDragHover] = useState(false);

    const [body, setBody] = useState("");
    const [focused, setFocused] = useState<boolean>(false);

    const [scanning, setScanning] = useState<boolean>(false);

    const [sending, setSending] = useState<string | null>(null);

    const createNote = useCreateNote(setSending);
    // const trySetAttachmentPath = useTrySetAttachmentPath(props.setAttachment);

    useEffect(() => {
      // this just determines whether to gray out send button
      const haveNote = body.length > 0;
      const attachmentOk = isValidNewAttachment(props.attachment, requireRender);
      setCanSend(
        !scanning &&
          ((haveNote && attachmentOk) ||
            (!haveNote && props.attachment != null && attachmentOk) ||
            (haveNote && props.attachment == null))
      );
    }, [body, props.attachment]);

    async function trySetAttachmentPath(file: string) {
      if (scanning) return;
      if (!props.setAttachment) return;

      invokeWithError(
        getAttachmentType(file).then((t) => {
          console.log(file);
          console.log(t);
          if ("Audio" in t) {
            props.setAttachment({ Audio: { title: null, path: file } });
          } else if ("Project" in t) {
            props.setAttachment({
              Project: { title: "", path: file, render: null },
            });
          }
        })
      );
    }

    useNoteBoxDragHandler(dragElems, setDragHover, trySetAttachmentPath);

    function addElemToDragSet(node: HTMLElement) {
      if (!dragElems.current.includes(node)) dragElems.current.push(node);
    }

    function onSend() {
      if (canSend) {
        createNote(props.channelUlid, body, props.replyUlid, props.attachment);
        setBody("");
        if (props.setAttachment) props.setAttachment(null);
        unfocusNoteBox();
        if (props.setReplyUlid) {
          props.setReplyUlid(null);
        }
        if (props.onSent) props.onSent();
      }
    }

    function unfocusNoteBox() {
      if (
        document.activeElement &&
        parentRef.current &&
        parentRef.current.contains(document.activeElement) &&
        document.activeElement instanceof HTMLElement
      ) {
        document.activeElement.blur();
      }
    }

    function focusTextarea() {
      if (
        document.activeElement &&
        textareaRef.current &&
        document.activeElement !== textareaRef.current
      ) {
        textareaRef.current.focus();
      }
    }

    function focusTextareaOnClick(e: MouseEvent, node: HTMLElement) {
      // checking that click was directory on given element
      if (e.target !== e.currentTarget) return;
      if (e.target instanceof HTMLElement && e.target === node) {
        e.preventDefault();
        e.stopPropagation();
        focusTextarea();
      }
    }

    const outerStyle =
      "relative text-sm border rounded-md ring-blue-500 transition-all duration-200 bg-white";
    const borderEnabled = "ring-2 ring-ring";
    const hoverStyle = "bg-blue-100 shadow-md";

    function renderHover() {
      if (dragHover && parentRef) {
        return (
          <div
            ref={(node) => {
              if (node) {
                addElemToDragSet(node);
              }
            }}
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col items-center justify-center"
          >
            <Plus />
            <p>Add attachment</p>
          </div>
        );
      }
    }

    function renderTextarea() {
      return (
        <NoteBoxTextarea
          ref={(node) => {
            if (node) {
              addElemToDragSet(node);
              textareaRef.current = node;
              node.onfocus = () => setFocused(true);
              node.onblur = () => setFocused(false);
            }
          }}
          body={body}
          setBody={setBody}
          onEnter={onSend}
          haveAttachment={props.attachment != null}
        />
      );
    }

    function renderReply() {
      if (props.replyUlid) {
        return (
          <NoteBoxReply
            channelUlid={props.channelUlid}
            replyUlid={props.replyUlid}
            {...(props.setReplyUlid && {
              onClearReply: () => props.setReplyUlid!(null),
            })}
          />
        );
      }
    }

    function renderAttachment() {
      if (props.attachment) {
        return (
          <div
            className="m-3"
            ref={(node) => {
              if (!node) return;
              node.onmousedown = (e) => focusTextareaOnClick(e, node);
            }}
          >
            <NewNoteAttachment
              attachment={props.attachment}
              setAttachment={props.setAttachment}
              setScanning={setScanning}
            />
          </div>
        );
      }
    }

    function renderSending() {
      return (
        <div className="flex p-2">
          <div className="ml-auto">
            <p className="flex items-center">
              {sending} <LoadingSpinnerBlack className="ml-2" />
            </p>
          </div>
        </div>
      );
    }

    function renderButtons() {
      return (
        <div
          className="flex p-2"
          ref={(node) => {
            if (!node) return;
            addElemToDragSet(node);
            node.onmousedown = (e) => focusTextareaOnClick(e, node);
          }}
        >
          <NoteBoxButtons
            ref={(node) => {
              if (node) {
                addElemToDragSet(node);
                node.onmousedown = (e) => focusTextareaOnClick(e, node);
              }
            }}
            onAttach={() => {
              (async () => {
                const path = await open({
                  multiple: false,
                  title: "Select a file to attach",
                  filters: [
                    {
                      extensions: ["als"],
                      name: "Ableton Live Set",
                    },
                    {
                      extensions: ["wav", "mp3", "flac", "ogg", "aif", "aiff"],
                      name: "Audio files",
                    },
                  ],
                });
                if (path) {
                  trySetAttachmentPath(path);
                  focusTextarea();
                }
              })();
            }}
            haveAttachment={props.attachment != null}
            onSend={onSend}
            canSend={canSend}
          />
        </div>
      );
    }

    return (
      <div
        ref={(node) => {
          if (node) {
            parentRef.current = node;
            addElemToDragSet(node);
            node.onmousedown = (e) => focusTextareaOnClick(e, node);
          }
        }}
        className={`absolute ${outerStyle} ${focused && borderEnabled} ${
          dragHover && hoverStyle
        }`}
      >
        {renderHover()}
        {renderTextarea()}
        {renderReply()}
        {renderAttachment()}
        {sending ? renderSending() : renderButtons()}
      </div>
    );
  }
);

function useNoteBoxDragHandler(
  elems: React.RefObject<HTMLElement[]>,
  setDragHover: (b: boolean) => void,
  trySetAttachmentPath: (path: string) => void
) {
  const addError = useStore((state) => state.addError);
  const lastDropTime = useRef<number | null>();

  useHandleTauriDrag((e) => {
    if (e.type == "leave") {
      setDragHover(false);
      return;
    }
    const isOver = elems.current!.some((elem) => {
      return checkWithin(e.position, elem);
    });
    if (!isOver) {
      // console.log("nope", e.position, elems);
      setDragHover(false);
      return;
    }
    switch (e.type) {
      case "enter":
      case "over":
        // console.log("yup", e.position, elems);
        setDragHover(true);
        break;
      case "drop":
        // drop debounce
        if (lastDropTime.current && Date.now() - lastDropTime.current < 200)
          return;
        lastDropTime.current = Date.now();
        setDragHover(false);
        if (e.paths.length != 1) {
          addError("Please drag and drop only one file at most.");
        } else {
          trySetAttachmentPath(e.paths[0]);
        }
        break;
    }
  });
}

function useCreateNote(
  setSending: (msg: string | null) => void
): (
  channelUlid: string | null,
  body: string,
  replyUlid: string | null,
  attachment: NewAttachment | null
) => void {
  const [folder, invokeWithError] = useStore((state) => [
    state.folder,
    state.invokeWithError,
  ]);

  return (
    channelUlid: string | null,
    body: string,
    replyUlid: string | null,
    attachment: NewAttachment | null
  ) => {
    (async () => {
      const msg: NewNote = {
        body,
        reply_ulid: replyUlid,
        attachment,
      };

      if (folder) {
        if (attachment && "Project" in attachment) {
          setSending("Adding note, this might take a second...");
        } else {
          setSending("Adding note...");
        }
        invokeWithError(createNote(folder, channelUlid, msg)).finally(() => {
          setSending(null);
        });
      }
    })();
  };
}

interface NoteBoxTextarea {
  body: string;
  setBody: (body: string) => void;
  onEnter: () => void;
  haveAttachment: boolean;
}

const NoteBoxTextarea = forwardRef<HTMLTextAreaElement, NoteBoxTextarea>(
  (props, ref) => {
    const textAreaRef = useRef<HTMLTextAreaElement | null>(null);
    useImperativeHandle(ref, () => textAreaRef.current!);

    return (
      <TextareaAutosize
        ref={textAreaRef}
        value={props.body}
        onChange={(e) => props.setBody(e.target.value)}
        onKeyDown={(e) => {
          if (e.key == "Enter" && !e.shiftKey) {
            e.preventDefault();
            props.onEnter();
          } else if (e.key == "Escape") {
            if (
              e.target instanceof HTMLTextAreaElement &&
              e.target == textAreaRef.current
            ) {
              e.target.blur();
            }
          }
        }}
        className="text-md border-none outline-none rounded-md p-3 resize-none bg-inherit w-full overflow-auto h-min-[100px] placeholder:text-muted-foreground"
        placeholder={
          props.haveAttachment
            ? "Write a note..."
            : "Write a note or drop an attachment here..."
        }
        maxRows={5}
      ></TextareaAutosize>
    );
  }
);

export function NoteBoxReply({
  channelUlid,
  replyUlid,
  onClearReply,
}: {
  channelUlid: string | null;
  replyUlid: string;
  onClearReply?: () => void;
}) {
  return (
    <div className="flex justify-between absolute bg-white rounded-lg border border-gray-200 translate-y-[-75px] w-full h-10 z-[-10]">
      <div className="ml-2">
        <ReplyText channelUlid={channelUlid} noteUlid={replyUlid} />
      </div>
      {onClearReply && (
        <div
          className="mr-2 ml-auto translate-y-1"
          onMouseDown={(e) => {
            e.preventDefault();
            onClearReply();
          }}
        >
          <X size={20} />
        </div>
      )}
    </div>
  );
}

interface NoteBoxButtonsProps {
  onAttach?: () => void;
  haveAttachment: boolean;

  onSend: () => void;
  canSend: boolean;
}

const NoteBoxButtons = forwardRef<HTMLDivElement, NoteBoxButtonsProps>(
  (props, ref) => {
    return (
      <div
        ref={ref}
        className="ml-auto"
        onMouseDown={(e) => {
          e.stopPropagation();
          e.preventDefault();
        }}
      >
        {props.onAttach && !props.haveAttachment && (
          <Button
            variant="ghost"
            size="icon"
            className="text-gray-400 hover:text-gray-600 transition-colors duration-200"
            onClick={props.onAttach}
          >
            <Paperclip className="h-5 w-5" />
          </Button>
        )}
        <Button
          size="icon"
          className={`${
            props.canSend
              ? "bg-orange-500 hover:bg-orange-600"
              : "bg-gray-400 hover:bg-gray-400"
          }text-white transition-all duration-300 ease-in-out`}
          onClick={props.onSend}
        >
          <Send className="h-5 w-5" />
        </Button>
      </div>
    );
  }
);
