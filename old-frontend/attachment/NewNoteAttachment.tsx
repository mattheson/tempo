import { NewAttachment } from "@bindings/NewAttachment";
import { AudioAttachment } from "./AudioAttachment";
import { ProjectAttachment } from "./ProjectAttachment";
import { produce } from "immer";
import { NewProjectAttachmentScan } from "./NewProjectAttachmentScan";
import { MutableAttachment } from "./Attachment";
import { NewProjectRender } from "./NewProjectRender";

export function NewNoteAttachment({
  attachment,
  setAttachment,
  setScanning,
}: {
  attachment: NewAttachment | null;
  setAttachment: (attachment: NewAttachment | null) => void;
  setScanning: (b: boolean) => void;
}) {
  function render() {
    if (attachment == null) return <></>;

    if ("Audio" in attachment) {
      return (
        <MutableAttachment
          title={attachment.Audio.title ? attachment.Audio.title : ""}
          setTitle={(title: string) => {
            if (title.length == 0) {
              setAttachment(
                produce(attachment, (draft) => {
                  draft.Audio.title = null;
                })
              );
            } else {
              setAttachment(
                produce(attachment, (draft) => {
                  draft.Audio.title = title;
                })
              );
            }
          }}
          titlePlaceholder="Add an optional title..."
          onXButton={() => setAttachment(null)}
        >
          <AudioAttachment filename={attachment.Audio.path} />
        </MutableAttachment>
      );
    }
    if ("Project" in attachment) {
      return (
        <MutableAttachment
          title={attachment.Project.title}
          setTitle={(title: string) => {
            setAttachment(
              produce(attachment, (draft) => {
                draft.Project.title = title;
              })
            );
          }}
          titlePlaceholder="Add a title..."
          onXButton={() => setAttachment(null)}
        >
          <ProjectAttachment
            projectPath={attachment.Project.path}
            projectType="Ableton project"
          >
            <div className="flex flex-row min-w-full items-center align-middle">
              <div className="mr-auto">
                <NewProjectRender
                  renderPath={attachment.Project.render}
                  setRenderPath={(path: string | null) => {
                    setAttachment(
                      produce(attachment, (draft) => {
                        draft.Project.render = path;
                      })
                    );
                  }}
                />
              </div>
              <div className="p-2">
                <NewProjectAttachmentScan
                  projectPath={attachment.Project.path}
                  setScanningProject={setScanning}
                />
              </div>
            </div>
          </ProjectAttachment>
        </MutableAttachment>
      );
    }
  }

  return render();
}
