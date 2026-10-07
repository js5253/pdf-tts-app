import { For } from "solid-js";
import { commands } from "../bindings"
import { ProgressBar } from "../components/ProgressBar";

export const DownloadManager = () => {
    const [downloadProgress, {refetch}] = commands.getModelDownloads();
    return(
        <div>
            <h1>Download Manager</h1>
            <For each={downloadProgress()}>
                {item => <li><ProgressBar progress={item.progress} /> </li>}
            </For>
        </div>
    )
}