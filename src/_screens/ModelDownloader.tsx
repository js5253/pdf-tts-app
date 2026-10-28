import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir } from "@tauri-apps/api/path"
import { createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { IoPlay } from "solid-icons/io";
import { commands } from "../bindings";

export const ModelDownloader = () => {
    const [downloadProgress, { refetch }] = commands.getDownloadManagerProgress();
    const onItemStop = () => {
        commands.stopModelDownload();
    }
    return (
        <div>
            <h1>Download Manager</h1>
            <For each={downloadProgress()}>
                {item => <li>
                    <span>{item.name}</span> - <ProgressBar progress={item.progress} /> -
                    <button><IoStop /></button></li>
                }
            </For>
        </div>
    )
}