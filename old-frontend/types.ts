import { ChannelDoc } from "@bindings/ChannelDoc";
import { NewAttachment } from "@bindings/NewAttachment";
import { RepliableComment } from "@bindings/RepliableComment";
import { TempoResult } from "@bindings/TempoResult";

export type Views = "home" | "folder";
export type FolderViews = "chat" | "tree" | "users";
export type CommentsType = { [key in string]?: RepliableComment };

// TODO add browser view

export function getChannelName(c: TempoResult<ChannelDoc>): string {
  if ("Ok" in c) {
    return (c.Ok).name;
  } else {
    return "Unknown name";
  }
}

export function getChannelDoc(c: TempoResult<ChannelDoc>): ChannelDoc | null {
  if ("Ok" in c) {
    return (c.Ok as ChannelDoc);
  } else {
    return null;
  }
}

export function isValidNewAttachment(a: NewAttachment | null, requireRender: boolean): boolean {
  if (!a) return true;

  if ("Audio" in a) {
    return a.Audio.title == null || a.Audio.title.length != 0;
  }

  if ("Project" in a) {
    const titleOk = a.Project.title.length != 0;
    if (requireRender) {
      return titleOk && a.Project.render != null;
    }
    return titleOk;
  }

  throw `isValidNewAttachment: found unexpected NewAttachment type ${a}`;
}
