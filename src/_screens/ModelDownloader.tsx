import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir } from "@tauri-apps/api/path"
import { createSignal, For, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { IoPlay, IoStop } from "solid-icons/io";
import { commands } from "../bindings";
import { ProgressBar } from "../components/ProgressBar";
import { translateSpectaToSolidResource } from "../util";

export const ModelDownloader = () => {

    const [downloadProgress] = translateSpectaToSolidResource(commands.getDownloadManagerProgress);
        console.log("DOWNMAN:", downloadProgress())
    const onItemStop = () => {
        commands.stopModelDownload();
    }
    return (
        <div>
            <h1 class="text-3xl">Download Manager</h1>
            <For each={downloadProgress()}>
                {item => <li class="bg-white-400 w-full">
                    <span>{item.name}</span> - <ProgressBar progress={item.progress!} /> -
                    <button><IoStop /></button></li>
                }
            </For>
            {downloadProgress()?.length > 0 && <><h2 class="font-xl text-gray-400">
                    No Downloads In Progress
                </h2></>
                }
        </div>
    )
}