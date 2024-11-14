import { SharedAttachment } from "@bindings/SharedAttachment";
import { SharedAudioAttachment } from "@bindings/SharedAudioAttachment";
import { SharedProjectAttachment } from "@bindings/SharedProjectAttachment";
import { useMemo } from "react";
import { Attachment } from "./Attachment";
import { ProjectInfo } from "@bindings/ProjectInfo";
import { AudioFileInfo } from "@bindings/AudioFileInfo";
import { ProjectAttachment } from "./ProjectAttachment";
import { TempoResult } from "@bindings/TempoResult";
import { CopyProjectButton } from "@/attachment/CopyProjectButton";
import { PlayAudioButton } from "./PlayAudioButton";
import { AudioAttachment } from "./AudioAttachment";

// attachment on a previously sent note
export function NoteAttachment({
  channelUlid,
  noteUlid,
  attachment,
}: {
  channelUlid: string | null;
  noteUlid: string;
  attachment: SharedAttachment;
}) {
  const [ty, att]: [
    "Project" | "Audio",
    SharedProjectAttachment | SharedAudioAttachment
  ] = useMemo(() => {
    if ("Project" in attachment) return ["Project", attachment.Project];
    return ["Audio", attachment.Audio];
  }, [attachment]);

  return ty == "Project" ? (
    <NoteProjectAttachment
      channelUlid={channelUlid}
      noteUlid={noteUlid}
      attachment={att as SharedProjectAttachment}
    />
  ) : (
    <NoteAudioAttachment attachment={att as SharedAudioAttachment} />
  );
}

function NoteProjectAttachment({
  channelUlid,
  noteUlid,
  attachment,
}: {
  channelUlid: string | null;
  noteUlid: string;
  attachment: SharedProjectAttachment;
}) {
  const [projectOk, projectErrOrInfo] = useMemo(() => {
    if ("Err" in attachment.project) return [false, attachment.project.Err];
    else return [true, attachment.project.Ok];
  }, [attachment]);

  return projectOk ? (
    <Attachment title={attachment.title}>
      <ProjectAttachment
        projectPath={(projectErrOrInfo as ProjectInfo).filename}
        projectType="Ableton project"
      >
        <div className="flex flex-row justify-end w-full items-center align-middle">
          <div className="mr-auto">
            {attachment.render && (
              <NoteProjectRender render={attachment.render} />
            )}
          </div>
          {projectOk ? (
            <CopyProjectButton
              channelUlid={channelUlid}
              noteUlid={noteUlid}
              title={attachment.title}
              projectData={(projectErrOrInfo as ProjectInfo).data}
            />
          ) : (
            <p className="text-red-500 m-4">
              <b>Error, your sync service might be syncing still:</b> {projectErrOrInfo as string}
            </p>
          )}
        </div>
      </ProjectAttachment>
    </Attachment>
  ) : (
    <p className="text-red-500 m-4">
      <b>
        Error while loading attachment, your sync service might still be syncing
        some files
      </b>
      : {projectErrOrInfo as string}
    </p>
  );
}

function NoteProjectRender({ render }: { render: TempoResult<AudioFileInfo> }) {
  const [ok, errOrInfo] = useMemo(() => {
    if ("Ok" in render) return [true, render.Ok];
    return [false, render.Err];
  }, [render]);

  return ok ? (
    <PlayAudioButton audio={errOrInfo as AudioFileInfo} />
  ) : (
    <p>
      <b>Error with render, your sync service might be syncing still: </b>
      {errOrInfo as string}
    </p>
  );
}

function NoteAudioAttachment({
  attachment,
}: {
  attachment: SharedAudioAttachment;
}) {
  const [ok, errOrInfo] = useMemo(() => {
    if ("Ok" in attachment.file) return [true, attachment.file.Ok];
    return [false, attachment.file.Err];
  }, [attachment]);
  return (
    <Attachment title={attachment.title}>
      {ok ? (
        <AudioAttachment filename={(errOrInfo as AudioFileInfo).filename}>
          <PlayAudioButton audio={errOrInfo as AudioFileInfo} />
        </AudioAttachment>
      ) : (
        <p>Error finding audio file: {errOrInfo as string}</p>
      )}
    </Attachment>
  );
}
