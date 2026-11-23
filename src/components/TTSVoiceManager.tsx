import { createEffect, createResource, createSignal, ErrorBoundary, For, onMount } from "solid-js";
import { commands, HasCompletedOnboarding } from "../bindings";
import { SHERPA_MODEL_ADDRESS, translateSpectaToSolidResource } from "../util";
import { IoArrowDown, IoDownload, IoInformationCircle, IoPlay, IoWarning } from "solid-icons/io";
import { Button } from "./Button";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { JSX } from "solid-js/h/jsx-runtime";

const languages = {
    "english": "en",
    "spanish": "es",
    "french": "fr"
}

export const TTSVoiceManager = () => {
    const [originalModels, { refetch }] = translateSpectaToSolidResource(commands.getOnlineModels);
    const [models, setModels] = createSignal(originalModels());
    const [language, setLanguage] = createSignal<undefined | string>();
    const [dataDir] = createResource<string>(appDataDir);
    const [isExpanded, setIsExpanded] = createSignal<boolean>(false);
    const [search, setSearch] = createSignal("");
    // const langs = new Map(Object.entries(languages))
    createEffect(() => {
        if (!originalModels()) return;
        setModels(originalModels())
    })
    const updateSearch: JSX.EventHandler<HTMLInputElement, KeyboardEvent> = (e) => {
        setSearch(e.currentTarget.value);
        if (!search() || search() == "") setModels(originalModels()) 

        setModels(originalModels()?.filter(model => model.name.includes(e.currentTarget.value)))
    }
    const updateLanguageFilter: JSX.EventHandler<HTMLSelectElement, MouseEvent> = (e) => {
        // if (!language()) return;
        setModels(originalModels()?.filter(model => model.name.includes("-" + languages[e.currentTarget.value])));
    }
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
                <div class="space-y-4 ">
                    <div class="flex flex-row items-center gap-2 rounded">
                        <label for="search">Search</label>
                        <input type="text" name="search" class="bg-white p-2" onChange={updateSearch} value={search()}/>
                        Language: <select onChange={updateLanguageFilter} value={language()}>
                            <option>All</option>
                            {Object.keys(languages).map(item => {
                                return <select>{item}</select>
                            }) }
                        </select>
                    </div>
                    <ul class="flex flex-col gap-2">
                        <For each={models()}>
                            {(item, idx) => !isExpanded() && idx() <= 5 ? <div class="flex flex-row items-between justify-between p-2 bg-green-50 rounded">{item.name} <button onClick={() => onDownloadRequested(item.id).then()}><IoDownload size={24} /></button></div> : null}
                        </For>
                        <Button className="p-2 bg-white flex flex-row items-center text-center" onClick={() => setIsExpanded(!isExpanded())}><IoArrowDown />Expand All</Button>
                    </ul>
                    <div>
                        <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
                    </div>

                </div>
            </ErrorBoundary>
        </div>
    )
}