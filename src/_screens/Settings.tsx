import { appDataDir } from "@tauri-apps/api/path"
import { openPath, openUrl } from "@tauri-apps/plugin-opener"
import { createResource, createSignal, For, onMount, Show } from "solid-js";
import { Button } from "../components/Button";
import classNames from "classnames";
import { commands, TtsAppConfig } from "../bindings";
import { SHERPA_MODEL_ADDRESS } from "../util";
const Settings = () => {
    const [models, {refetch: refetchModels}] = createResource<string[]>(commands.getDownloadedModels);
    const [defaultSettings, {refetch: refetchSettings}] = createResource<string[]>(commands.getDownloadedModels);
    const [dataDir] = createResource<string>(appDataDir)
    const [settings, setSettings] = createSignal<TtsAppConfig | null>(defaultSettings());
    const reloadTtsModels = async () => {
        await refetchModels()
    }
    const reloadSettings = async () => {
        await refetchSettings()
    }

    const makeDefaultModel = async (modelName: string) => {
        await commands.setDefaultModel(modelName);
        await reloadSettings();
        await reloadTtsModels();
    }
    const onUpdateSetting = (key: string, value: any) => {
        setSettings(prev => ({...prev, [key]: value}))
    }

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
                    <Button className="p-2 bg-blue-400" onClick={() => openUrl(SHERPA_MODEL_ADDRESS)}>Open Sherpa-ONNX models</Button>
                    <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Open Directory</Button>
                    <Button onClick={reloadTtsModels}>Reload TTS Models</Button>
                </div>
                <ul class="flex flex-col gap-2">
                    <For each={models()}>
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