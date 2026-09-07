import { invoke } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path"
import { openUrl } from "@tauri-apps/plugin-opener"
import { createSignal, onMount } from "solid-js";
import { Button } from "../components/Button";

const Settings = () => {
    const [dataDir, setDataDir] = createSignal<string | null>(null);
    const [downloadedTtsModels, setDownloadedTtsModels] = createSignal<string[] | null>(null);
    const reloadTtsModels = async () => {
        setDownloadedTtsModels(await invoke('get_downloaded_models'))

    }
    onMount(async () => {
        console.log(await appDataDir())
        setDataDir(await appDataDir())
        await reloadTtsModels()
    })
    return (
        <>
            <div class="flex-col gap-2">
                <h1 class="text-xl">Default Options</h1>
                <input type="checkbox" name="sameOutputDirectory" />
                <label for="sameOutputDirectory">Put the output files in the same directory as the input file.</label>
                <input type="text" name="voice" />
                <label for="voice">Voice</label>
                <input type="number" name="speed" />
                <label for="speed">Speed</label>
                <input type="checkbox" name="combinePages" />
                <label for="combinePages">Combine Pages</label>
                <input type="number" name="speakerId" />
                <label for="speakerId">Speaker ID</label>
            </div>
            <div class="flex flex-col gap-2">
                <h2 class="text-xl">TTS Voice Manager</h2>
                <p>Currently, downloading TTS voices is not available in-app. Manually download models, unzip them, and place them into the app's location/tts folder.</p>
                <div class="gap-4 flex flex-row">
                    <Button className="p-2 bg-blue-400" onClick={() => { openUrl('https://github.com/k2-fsa/sherpa-onnx').then() }}>Open Sherpa-ONNX models</Button>
                    <Button className="p-2 bg-blue-400" onClick={() => openUrl(dataDir())}>Open Directory</Button>
                    <Button className="p-2 bg-blue-400" onClick={reloadTtsModels}>Reload TTS Models</Button>
                </div>
                <ul>
                    {downloadedTtsModels()?.map(model => (
                        <li class="px-4 py-2 bg-gray-200 rounded">{model}</li>
                    ))}
                </ul>

            </div>
        </>
    )
}
export { Settings }