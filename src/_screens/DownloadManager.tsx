import { For } from "solid-js";
import { commands } from "../bindings"
import { ProgressBar } from "../components/ProgressBar";
import { IoStop } from "solid-icons/io";

export const DownloadManager = () => {
    const [downloadProgress, { refetch }] = commands.getModelDownloads();
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