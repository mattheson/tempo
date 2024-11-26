// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ChannelData } from "./ChannelData";
import type { SharedNote } from "./SharedNote";
import type { TempoResult } from "./TempoResult";

/**
 * All data stored in a folder. Sent to the frontend.
 */
export type FolderData = { username: string, global: { [key in string]?: TempoResult<SharedNote> }, channels: { [key in string]?: ChannelData }, };