import { useEffect, useRef, useState } from "react";
import { useStore } from "./Store";
import { convertFileSrc } from "@tauri-apps/api/core";
import AudioPlayer from "react-h5-audio-player";
import "react-h5-audio-player/lib/styles.css";

export function Player() {
  const [playingInfo, playing] = useStore((state) => [
    state.playingInfo,
    state.playing,
  ]);

  const [src, setSrc] = useState<string | null>(null);

  const audioRef = useRef<AudioPlayer | null>(null);

  useEffect(() => {
    if (playingInfo) {
      setSrc(convertFileSrc(playingInfo.path));
    } else {
      setSrc(null);
    }
  }, [playingInfo]);

  useEffect(() => {
    if (playing) audioRef.current?.audio.current?.play();
    else audioRef.current?.audio.current?.pause();
  }, [playing]);

  return (
    <AudioPlayer
      src={src ? src : ""}
      className="min-w-max min-h-max"
      layout="stacked-reverse"
      ref={audioRef}
      {...(src
        ? { customAdditionalControls: [<div>{playingInfo?.filename}</div>] }
        : { customAdditionalControls: [] })}
      // customAdditionalControls={[
      //   renderInfoText()
      // ]}
    />
  );
}
