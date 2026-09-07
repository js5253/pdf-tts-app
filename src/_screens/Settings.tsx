import { appDataDir } from "@tauri-apps/api/path"
import { openUrl } from "@tauri-apps/plugin-opener"
import { createSignal, onMount } from "solid-js";

const Settings = () => {
    const [dataDir, setDataDir] = createSignal<string | null>(null);
    onMount(async () => {
        setDataDir(await appDataDir())
    })
    return (
        <>
            <div class="flex-col">
                <h1>Default Options</h1>
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
            <div>
                Download Manager
                <p>Currently, downloading TTS voices is not available in-app. Check <a target="blank" href="">here</a> to download TTS models, and unzip/put them <a href="#">here.</a></p>
            </div>
            <div>
                <button onClick={() => { openUrl('https://github.com/k2-fsa/sherpa-onnx').then() }}>Open Sherpa-ONNX models</button>
                <button onClick={() => openUrl(dataDir())}>Open Directory</button>
            </div>
        </>
    )
}
export { Settings }