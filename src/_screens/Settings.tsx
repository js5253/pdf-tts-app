import { appDataDir } from "@tauri-apps/api/path"
import { openPath, openUrl } from "@tauri-apps/plugin-opener"
import { createResource, For, Show } from "solid-js";
import { Button } from "../components/Button";
import classNames from "classnames";
import { commands, } from "../bindings";
import { SHERPA_MODEL_ADDRESS, translateSpectaToSolidResource } from "../util";
import { TTSVoiceManager } from "../components/TTSVoiceManager";
import { IoInformation, IoInformationCircle } from "solid-icons/io";
const Settings = () => {
    const [config] = translateSpectaToSolidResource(commands.getConfig);
    const [models, { refetch: refetchModels }] = translateSpectaToSolidResource(commands.getDownloadedModels)
    const [dataDir] = createResource<string>(appDataDir)
    const reloadTtsModels = async () => {
        await refetchModels()
    }
    const reloadSettings = async () => {
        // await refetchSettings()
    }

    const makeDefaultModel = async (modelName: string) => {
        await commands.setDefaultModel(modelName);
        await reloadSettings();
        await reloadTtsModels();
    }
    // const onUpdateSetting = (key: string, value: any) => {
    //     setSettings(prev => ({...prev, [key]: value}))
    // }

    return (
        <div class="w-[800px] bg-green-50 p-4 space-y-4">{config ? <>
            <div class="flex-col gap-2">
                <Button onClick={() => openPath(dataDir() + '/out')}>Open Output Directory</Button>
                <h1 class="text-xl">Default Options</h1>
                <div>
                    <input type="checkbox" name="sameOutputDirectory" />
                    <label for="sameOutputDirectory">Put the output files in the same directory as the input file.</label>
                </div>
                <div>
                    <input type="number" name="speed" value={config()?.speed!} />
                    <label for="speed">Speed</label>
                </div>
                <div>
                    <input type="checkbox" name="combinePages" value={String(config()?.combine_pages)} />
                    <label for="combinePages">Combine Pages</label>
                </div>
                <div>
                    <input type="number" name="speakerId" value={config()?.speaker_id} />
                    <label for="speakerId">Speaker ID</label>
                </div>
                <Button onClick={() => { }}>Save</Button>
            </div>
            <div class="flex flex-col gap-2">
                <h2 class="text-xl">TTS Voice Manager</h2>
                <ul class="flex flex-col gap-2">
                    <h3>Downloaded</h3>
                    <For each={models()} fallback={<p class="flex flex-row items-cente gap-1"><IoInformationCircle />No Models Downloaded.</p>}> 
                        {(model) => <li data-index={model} class={classNames("shadow bg-green-50 px-4 py-2 bg-gray-200 rounded flex flex-row items-between justify-between", { "font-bold": config().voice == model })}>{model}

                            <Show when={config().voice !== model}><Button onClick={() => makeDefaultModel(model)}>Make Default</Button></Show></li>}
                    </For>
                    <h3>Get TTS Voices</h3>
                </ul>
            </div> </> : <p></p>}
            <TTSVoiceManager />
        </div>
    )
}
export { Settings }