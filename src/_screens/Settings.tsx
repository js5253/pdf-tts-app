import { invoke } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path"
import { openPath, openUrl } from "@tauri-apps/plugin-opener"
import { createSignal, For, onMount, Show } from "solid-js";
import { Button } from "../components/Button";
import classNames from "classnames";
import { commands, TtsAppConfig } from "../bindings";
const SHERPA_MODEL_ADDRESS = "https://k2-fsa.github.io/sherpa/onnx/tts/all/";
const Settings = () => {
    const [dataDir, setDataDir] = createSignal<string | null>(null);
    const [settings, setSettings] = createSignal<TtsAppConfig | null>(null);
    const [downloadedTtsModels, setDownloadedTtsModels] = createSignal<string[] | null>(null);
    const reloadTtsModels = async () => {
        const models = await commands.getDownloadedModels();
        if (models.status === "error") throw new Error();
        setDownloadedTtsModels(models.data);

    }
    const reloadSettings = async () => {
        const sett = await commands.getConfig();
        if (sett.status === 'error') throw new Error();
        setSettings(sett.data)

    }
    const makeDefaultModel = async (modelName: string) => {
        await commands.setDefaultModel(modelName);
        await reloadSettings();
        await reloadTtsModels();
    }
    onMount(async () => {
        await reloadSettings();
        console.log(await appDataDir())
        setDataDir(await appDataDir())
        await reloadTtsModels()
    })
    return (
        <>{settings ? <>
            <div class="flex-col gap-2">
                <h1 class="text-xl">Default Options</h1>
                <div>
                <input type="checkbox" name="sameOutputDirectory" />
                <label for="sameOutputDirectory">Put the output files in the same directory as the input file.</label>
                </div>
                <div>
                <input type="number" name="speed" value={settings()?.speed!} />
                <label for="speed">Speed</label>
                </div>
                <div>
                <input type="checkbox" name="combinePages" value={String(settings()?.combine_pages)}/>
                <label for="combinePages">Combine Pages</label>
                </div>
                <div>
                <input type="number" name="speakerId" value={settings()?.speaker_id}/>
                <label for="speakerId">Speaker ID</label>
                </div>
                <Button onClick={null}>Save</Button>
            </div>
            <div class="flex flex-col gap-2">
                <h2 class="text-xl">TTS Voice Manager</h2>
                <p>Currently, downloading TTS voices is not available in-app. Manually download models, unzip them, and place them into the app's location/tts folder. NOTE: only VITS TTS models are supported at the moment.</p>
                <div class="gap-2 flex flex-row">
                    <Button className="p-2 bg-blue-400" onClick={() => { openUrl(SHERPA_MODEL_ADDRESS).then() }}>Open Sherpa-ONNX models</Button>
                    <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir())}>Open Directory</Button>
                    <Button onClick={reloadTtsModels}>Reload TTS Models</Button>
                </div>
                <ul class="flex flex-col gap-2">
                    <For each={downloadedTtsModels()}>
                        {(model) => <li data-index={model} class={classNames("shadow bg-green-50 px-4 py-2 bg-gray-200 rounded flex flex-row items-between justify-between", { "font-bold": settings().voice == model })}>{model}

                            <Show when={settings().voice !== model}><Button onClick={() => makeDefaultModel(model)}>Make Default</Button></Show></li>}
                    </For>
                </ul>
            <Button>Open Output Directory</Button>
            </div> </> : <p></p>}
        </>
    )
}
export { Settings }