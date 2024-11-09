import { truncateAttachmentTitle } from "../misc";
import { XButton } from "@/XButton";
import { useState, useRef, useEffect, ReactNode } from "react";

// base of attachments

// immutable attachment
export function Attachment({
  title,
  children,
}: {
  title: string | null;
  children?: ReactNode;
}) {
  return (
    <div className="flex flex-col shadow-md rounded-lg bg-white border-gray-300 relative border m-2">
      <div className="flex flex-col">
        {title && (
          <h1 className="text-2xl font-bold m-2 rounded">
            {truncateAttachmentTitle(title)}
          </h1>
        )}
        {children}
      </div>
    </div>
  );
}

// attachment with an editable title
export function MutableAttachment({
  title,
  setTitle,
  titlePlaceholder,

  onXButton,

  children,
}: {
  title: string;
  setTitle: (title: string) => void;
  titlePlaceholder: string;

  onXButton: () => void;

  children?: ReactNode;
}) {
  const [isEditingTitle, setIsEditing] = useState(false);

  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (isEditingTitle && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isEditingTitle]);

  function handleTitleClick() {
    setIsEditing(true);
  }

  function handleTitleChange(e: React.ChangeEvent<HTMLInputElement>) {
    setTitle!(e.target.value);
    setIsEditing(true);
  }

  function handleBlur() {
    if (title.trim() !== "") {
      setIsEditing(false);
    }
  }

  return (
    <div className="flex flex-col shadow-md rounded-lg bg-white border-gray-300 relative border">
      <XButton onClick={onXButton} />
      <div className="flex flex-col">
        {isEditingTitle || title.length == 0 ? (
          <input
            ref={inputRef}
            value={title}
            onChange={handleTitleChange}
            onBlur={handleBlur}
            className="text-2xl font-bold m-2 focus:outline-none"
            placeholder={titlePlaceholder}
          />
        ) : (
          <h1
            onClick={handleTitleClick}
            className="text-2xl font-bold m-2 rounded transition-colors cursor-pointer hover:bg-gray-100"
          >
            {title}
          </h1>
        )}
        {children}
      </div>
    </div>
  );
}
