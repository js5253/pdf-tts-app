import { createResource, ErrorBoundary, For, onMount } from "solid-js";
import { commands, HasCompletedOnboarding } from "../bindings";
import { SHERPA_MODEL_ADDRESS, translateSpectaToSolidResource } from "../util";
import { IoDownload, IoInformationCircle, IoPlay, IoWarning } from "solid-icons/io";
import { Button } from "./Button";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";

export const TTSVoiceManager = () => {
    const [models, { refetch }] = translateSpectaToSolidResource(commands.getOnlineModels);
    const [dataDir] = createResource<string>(appDataDir);

    const onDownloadRequested = async (id: number) => {
        await commands.downloadModel(id);
    }
    const reloadTtsModels = async () => {
        await refetch()
        console.log(models())
    }
    return (
        <div class="bg-green-100 p-4 backdrop-filter-12 rounded">
        <ErrorBoundary fallback={<div class="flex flex-col">
            <p class="flex flex-row align-center gap-1"> <IoWarning />You're offline. Set TTS up later, or copy your model directly to the folder.</p>
            <div class="flex flex-row gap-2">
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Open TTS Directory</Button>
                <Button className="p-2 bg-blue-400" onClick={() => openUrl(SHERPA_MODEL_ADDRESS)}>Open Sherpa-ONNX models</Button>
                <Button onClick={reloadTtsModels}>Reload TTS Models</Button>


            </div>
        </div>}>
            <div class="space-y-4">
                <p>Let's set things up. Select a TTS model to download.</p>
                Language: <select>
                    <option>All</option>
                    <option>English</option>
                    <option>Spanish</option>
                    <option></option>
                </select>
                <ul class="flex flex-col gap-2">
                    <For each={models()}>
                        {(item) => <div class="flex flex-row items-between justify-between p-2 bg-green-50 rounded">{item.name} <button onClick={() => onDownloadRequested(item.id).then()}><IoDownload size={24} /></button></div>}
                    </For>
                </ul>
                <div>
                    <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
                </div>

            </div>
        </ErrorBoundary>
        </div>
    )
}