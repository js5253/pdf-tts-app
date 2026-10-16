import { createResource, ErrorBoundary, For, onMount } from "solid-js";
import { commands, HasCompletedOnboarding } from "../bindings";
import { translateSpectaToSolidResource } from "../util";
import { IoDownload, IoPlay } from "solid-icons/io";
import { Button } from "./Button";
import { openPath } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { Channel } from "@tauri-apps/api/core";
import { useEventContext } from "../context/EventContext";

export const TTSVoiceManager = () => {
    const [models, { refetch }] = translateSpectaToSolidResource(commands.getOnlineModels);
    const [dataDir] = createResource<string>(appDataDir);
    console.log(models())
    return (
        <ErrorBoundary fallback={<div class="flex flex-col">
            <p>You're offline. Set TTS up later, or copy your model directly to the folder.</p>
            <div>
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
            </div>
            <Button onClick={() => refetch()}>Reload</Button>
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
                        {(item) => <div class="flex flex-row items-between justify-between p-2 bg-green-50 rounded">{item.name} <button><IoDownload size={24} /></button></div>}
                    </For>
                </ul>
                <div>
                    <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
                </div>

            </div>
        </ErrorBoundary>
    )
}