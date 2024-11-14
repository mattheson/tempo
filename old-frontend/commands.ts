import { invoke } from "@tauri-apps/api/core";
import { NewNote } from "@bindings/NewNote";
import { NewComment } from "@bindings/NewComment";
import { FolderInfo } from "@bindings/FolderInfo";
import { FolderData } from "@bindings/FolderData";
import { ChannelDoc } from "@bindings/ChannelDoc";
import { TempoResult } from "@bindings/TempoResult";
import { SharedNote } from "@bindings/SharedNote";
import { SharedFileInfo } from "@bindings/SharedFileInfo";
import { ProjectFileRefScan } from "@bindings/ProjectFileRefScan";
import { PluginScan } from "@bindings/PluginScan";
import { FileErr } from "@bindings/FileErr";
import { AttachmentType } from "@bindings/AttachmentType";
import { useStore } from "./Store";

export type InvokePromise<T> = Promise<T>;

function pollFoldersOnce() {
  return useStore.getState().pollFoldersOnce();
}

function pollFolderDataOnce() {
  return useStore.getState().pollFolderDataOnce();
}

export async function getStorePath(): InvokePromise<string> {
  return invoke<string>("get_store_path");
}

export async function needFullDisk(): InvokePromise<boolean> {
  return invoke<boolean>("need_full_disk");
}

export async function openFullDisk(): InvokePromise<null> {
  return invoke<null>("open_full_disk");
}

export async function restart(): InvokePromise<null> {
  return invoke<null>("restart");
}

export async function fatal(msg: string): InvokePromise<null> {
  return invoke<null>("fatal", { msg });
}

export async function verifyUserHasAbleton(): InvokePromise<boolean> {
  return invoke<boolean>("verify_user_has_ableton");
}

export async function scanFolders(): InvokePromise<FolderInfo[]> {
  return invoke<FolderInfo[]>("scan_folders");
}

export async function scanFolder(folder: string): InvokePromise<FolderInfo> {
  return invoke<FolderInfo>("scan_folder", { folder });
}

export async function getFolderData(folder: string): InvokePromise<FolderData> {
  return invoke<FolderData>("get_folder_data", { folder });
}

export async function checkFolderInsideFolder(folder: string): InvokePromise<string | null> {
  return invoke<string | null>("check_folder_inside_folder", { folder });
}

export async function isUsernameFree(folder: string, username: string): InvokePromise<boolean> {
  return invoke<boolean>("is_username_free", { folder, username });
}

export async function scanPlugins(): InvokePromise<null> {
  return invoke<null>("scan_plugins");
}

export async function getLastPluginScanTime(): InvokePromise<number | null> {
  return invoke<number | null>("get_last_plugin_scan_time");
}

export async function createOrAddFolder(folder: string, username: string): InvokePromise<null> {
  return invoke<null>("create_or_add_folder", { folder, username }).then(() => { pollFoldersOnce(); return null; });
}

export async function removeFolder(folder: string): InvokePromise<null> {
  return invoke<null>("remove_folder", { folder }).then(() => { pollFoldersOnce(); return null; });
}

export async function createChannel(folder: string, channelName: string): InvokePromise<ChannelDoc> {
  return invoke<ChannelDoc>("create_channel", { folder, channelName }).then((d) => { pollFolderDataOnce(); return d; });
}

export async function getAttachmentType(file: string): InvokePromise<AttachmentType> {
  return invoke<AttachmentType>("get_attachment_type", { file });
}

// TODO this should always be Ok?
export async function createNote(
  folder: string,
  channelUlid: string | null,
  note: NewNote
): InvokePromise<TempoResult<SharedNote>> {
  return invoke<TempoResult<SharedNote>>("create_note", { folder, channelUlid, note }).then((d) => { pollFolderDataOnce(); return d; });
}

export async function addComment(folder: string, channelUlid: string | null, noteUlid: string, comment: NewComment): InvokePromise<TempoResult<SharedNote>> {
  return invoke<TempoResult<SharedNote>>("add_comment", { folder, channelUlid, noteUlid, comment }).then((d) => { pollFolderDataOnce(); return d; });
}

export async function copyProject(folder: string, channelUlid: string | null, noteUlid: string, destDir: string): InvokePromise<[String, FileErr[]]> {
  return invoke<[String, FileErr[]]>("copy_project", { folder, channelUlid, noteUlid, destDir });
}

export async function getFileInfo(folder: string, fileSha256: string): InvokePromise<SharedFileInfo> {
  return invoke<SharedFileInfo>("get_file_info", { folder, fileSha256 });
}

export async function scanProjectFileRefs(project: String): InvokePromise<ProjectFileRefScan> {
  return invoke<ProjectFileRefScan>("scan_project_file_refs", { project });
}

export async function scanProjectPlugins(folder: string, project: String): InvokePromise<PluginScan> {
  return invoke<PluginScan>("scan_project_plugins", { folder, project });
}
