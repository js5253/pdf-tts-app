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
    const onItemStop = (id: string) => {
        commands.stopModelDownload(id);
    }
    return (
        <div>
            <h1 class="text-4xl">Jobs</h1>
            <div class="w-full text-left">
                <h2 class="text-xl">Download Progress</h2>
                <ul class="w-full">
                    {!downloadProgress() || downloadProgress()?.length == 0 && <>No Model Downloads Ongoing...</>}
                    <For each={downloadProgress()}>
                        {item => (
                            item.type = "inProgress" ? <li>
                            <span>{item.name}</span> - <ProgressBar progress={item.progress!} /> -
                            <button onClick={() => onItemStop(item.name)}><IoStop /></button>
                            </li> : <li>
                            <span>{item.name}</span> - <span>Completed</span> -
                            <button onClick={() => onItemStop(item.name)}><IoStop /></button></li>
    )}
                        {/* {item => <li class="bg-white-400 w-full">
                        } */}

                    </For>
                    {/* {!downloadProgress() || downloadProgress()?.length == 0 && <>No Model Downloads Ongoing...</>}
                            {inProgressJobs()?.map(item => <TTSCard item={item} />)}
                        </ul>
*/}
                </ul>
                <h2 class="text-xl">Done</h2>

            </div>
        </div>
    )
}