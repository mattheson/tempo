import { Button } from "@/components/ui/button";
import { truncateAttachmentThing } from "@/misc";
import { useStore } from "@/Store";
import { AudioFileInfo } from "@bindings/AudioFileInfo";
import { Pause, Play } from "lucide-react";

export function PlayAudioButton({ audio }: { audio: AudioFileInfo }) {
  const [playing, playingInfo, setPlayingInfo, setPlaying] = useStore(
    (state) => [
      state.playing,
      state.playingInfo,
      state.setPlayingInfo,
      state.setPlaying,
    ]
  );

  return (
    <Button
      onClick={() => {
        if (!playingInfo) {
          setPlayingInfo(audio);
          setPlaying(true);
        } else if (playingInfo) {
          if (playingInfo.path != audio.path) {
            setPlayingInfo(audio);
            setPlaying(true);
          } else {
            setPlaying(!playing);
          }
        }
      }}
      className="rounded-2xl bg-gray-500 py-8"
    >
      <div className="flex flex-col items-center align-middle my-5">
        {!playing || (playingInfo && playingInfo.path != audio.path) ? (
          <Play fill="white" />
        ) : (
          <Pause fill="white" />
        )}
        {truncateAttachmentThing(audio.filename)}
      </div>
    </Button>
  );
}
