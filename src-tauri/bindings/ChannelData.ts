// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ChannelDoc } from "./ChannelDoc";
import type { SharedNote } from "./SharedNote";
import type { TempoResult } from "./TempoResult";

/**
 * All data stored in a folder. Sent to the frontend.
 */
export type ChannelData = { meta: TempoResult<ChannelDoc>, notes: { [key in string]?: TempoResult<SharedNote> }, };
