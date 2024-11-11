// comments field under project
// just in project modal for now

import { useMemo, useRef, useState } from "react";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import {
  ChevronDown,
  ChevronUp,
  MessageCircle,
  Reply,
  Send,
  X,
} from "lucide-react";
import { addComment } from "../commands";
import { useStore } from "../Store";
import { CommentsType } from "@/types";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Comment } from "@bindings/Comment";
import { decodeTime } from "ulidx";

// TODO this copies a lot of code from NoteBox and Note
// would be nice to reuse code here

// dropdown, keep height fixed
// text input on bottom
export function Comments({
  channelUlid,
  noteUlid,
  comments,
}: {
  channelUlid: string | null;
  noteUlid: string;
  comments: CommentsType;
}) {
  const [folder, invokeWithError] = useStore((state) => [
    state.folder!,
    state.invokeWithError,
  ]);

  const [newComment, setNewComment] = useState<string>("");
  const [expanded, setExpanded] = useState<boolean>(false);
  const [replyUlid, setReplyUlid] = useState<string | null>(null);

  const textAreaRef = useRef<HTMLTextAreaElement | null>(null);

  const canSend = useMemo(() => newComment.trim().length > 0, [newComment]);

  function tryAddComment() {
    if (!canSend) return;
    invokeWithError(
      addComment(folder, channelUlid, noteUlid, {
        reply_ulid: replyUlid,
        body: newComment,
      })
    );
    setNewComment("");
    setReplyUlid(null);
    textAreaRef.current?.blur();
  }

  function renderToggle() {
    return (
      <div
        className="flex flex-row mx-2 my-1"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="mr-3 font-bold text-lg">
          {Object.entries(comments).length}
        </div>
        <MessageCircle fill="black" />
        <div className="ml-auto">
          {expanded ? <ChevronUp /> : <ChevronDown />}
        </div>
      </div>
    );
  }

  function renderExpanded() {
    return (
      <div className="m-2 relative">
        <CommentsList
          comments={comments}
          onReply={(ulid) => setReplyUlid(ulid)}
        />

        {replyUlid && (
          <CommentReply
            sender={comments[replyUlid]!.comment.sender}
            text={comments[replyUlid]!.comment.body}
            onClearReply={() => setReplyUlid(null)}
          />
        )}

        <div className="relative z-[2]">
          <Textarea
            ref={textAreaRef}
            value={newComment}
            className="resize-none mb-2"
            onChange={(e) => {
              setNewComment(e.target.value);
            }}
            onKeyDown={(e) => {
              if (e.key == "Enter" && !e.shiftKey) {
                tryAddComment();
                e.preventDefault();
              }
            }}
          />
          <div className="flex flex-col">
            <Button
              size="icon"
              className={`ml-auto ${
                canSend
                  ? "bg-orange-500 hover:bg-orange-600"
                  : "bg-gray-400 hover:bg-gray-400"
              }text-white transition-all duration-300 ease-in-out`}
              onClick={() => {
                if (canSend) tryAddComment();
              }}
            >
              <Send className="h-5 w-5" />
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col shadow-md rounded-lg bg-white border-gray-300 relative border m-2">
      {renderToggle()}
      {expanded && renderExpanded()}
    </div>
  );
}

function CommentReply({
  sender,
  text,
  onClearReply,
}: {
  sender: string;
  text: string;
  onClearReply: () => void;
}) {
  return (
    <div className="flex justify-between absolute bg-white rounded-lg border border-gray-200 translate-y-[-2em] w-full h-10 px-2 z-[1] text-sm">
      <p>
        Replying to <b>{sender}:</b>{" "}
        {text.length > 30 ? text.slice(0, 30) + "..." : text}
      </p>
      <div
        className="translate-y-1"
        onMouseDown={(e) => {
          e.preventDefault();
          onClearReply();
        }}
      >
        <X size={20} />
      </div>
    </div>
  );
}

function CommentsList({
  comments,
  onReply,
}: {
  comments: CommentsType;
  onReply: (commentUlid: string) => void;
}) {
  return (
    <ScrollArea className="max-h-72 min-h-fit my-2 text-sm">
      {Object.entries(comments).sort().map(([ulid, comment]) => (
        <div key={ulid}>
          <AComment
            commentUlid={ulid}
            comment={comment!.comment}
            onReply={() => {
              onReply(ulid);
            }}
          />
          {Object.entries(comment!.replies).map(([ulid, comment]) => (
            <AComment
              key={"rply" + ulid}
              commentUlid={ulid}
              comment={comment!}
            />
          ))}
        </div>
      ))}
    </ScrollArea>
  );
}

function AComment({
  commentUlid,
  comment,
  onReply,
}: {
  commentUlid: string;
  comment: Comment;
  onReply?: () => void;
}) {
  const [replyMousedOver, setReplyMousedOver] = useState<boolean>(false);

  return (
    <div
      className={` ${
        !onReply && "ml-16"
      } flex flex-col relative transition-all duration-300 ease-in-out shadow-md rounded-lg bg-white border border-gray-200 h-full w-full my-2`}
    >
      <div className="flex justify-between items-center p-2">
        <div>
          <p className="text-wrap select-none mr-3">
            <b className="mr-2">{comment.sender}</b>
            {new Date(decodeTime(commentUlid)).toLocaleString("en-US", {
              year: "numeric",
              month: "short",
              day: "numeric",
              hour: "2-digit",
              minute: "2-digit",
            })}
          </p>
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
      <p className="p-3">{comment.body}</p>
    </div>
  );
}

/*
export function Comments({
  noteUlid
} : {}) {
  const [folders, folder, channelUlid, noteUlid, invokeWithError] = useStore((state) => [
    state.folders,
    state.folder!,
    state.channelUlid,
    state.noteUlid!,
    state.invokeWithError
  ]);

  const [newComment, setNewComment] = useState("");
  const canSend = newComment.trim().length > 0;
  const textAreaRef = useRef<HTMLTextAreaElement | null>(null);

  const note = channelUlid
    ? folders[folder]?.channels[channelUlid]?.notes[noteUlid]
    : folders[folder]?.global[noteUlid];

  function handleAddComment() {
    if (!canSend) return;
    invokeWithError(addComment(folder, channelUlid, noteUlid, { reply_ulid: null, body: newComment }));
    setNewComment("");
    if (textAreaRef.current) textAreaRef.current.value = "";
  }

  return note.project ? (
    <div>
      <h1 className="text-xl py-2">Comments</h1>
      <div className="max-h-40 overflow-scroll mb-5">
        {Object.keys(note.project.comments).length > 0 ? (
          Object.entries(note.project.comments)
            .sort()
            .map(([key, value]) => <p key={key}>{JSON.stringify(value)}</p>)
        ) : (
          <i className="text-gray-600">No comments yet...</i>
        )}
      </div>
      <Textarea
        className="h-15 resize-none"
        onChange={(e) => setNewComment(e.target.value)}
        ref={textAreaRef}
        onKeyDown={(e) => {
          if (e.key == "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleAddComment();
          }
        }}
      />
      <div className="flex p-2">
        <div className="ml-auto">
          <Button
            size="icon"
            className={`${
              canSend
                ? "bg-orange-500 hover:bg-orange-600"
                : "bg-gray-400 hover:bg-gray-400"
            }text-white transition-all duration-300 ease-in-out`}
            onClick={handleAddComment}
          >
            <Send className="h-5 w-5" />
          </Button>
        </div>
      </div>
    </div>
  ) : (
    <p className="text-red-300">
      Error: note does not have a project attached
    </p>
  );
}
*/
