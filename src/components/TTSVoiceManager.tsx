import { createResource, ErrorBoundary, For } from "solid-js";
import { commands, HasCompletedOnboarding } from "../bindings";
import { translateSpectaToSolidResource } from "../util";
import { IoPlay } from "solid-icons/io";
import { Button } from "./Button";
import { openPath } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { Channel } from "@tauri-apps/api/core";

export const TTSVoiceManager = () => {
    const [models, { refetch }] = translateSpectaToSolidResource(commands.getOnlineModels);
    const [dataDir] = createResource<string>(appDataDir);
    const onEvent = new Channel<HasCompletedOnboarding>(); // this is probably an erroneous line
    onEvent.onmessage = async (_) => {
        await refetch();
    }

    const exitCompletedOnboarding = async () => {
        await commands.setCompletedOnboarding(true, onEvent);
    }
    return (
        <ErrorBoundary fallback={<div class="flex flex-col">
            <p>You're offline. Set TTS up later, or copy your model directly to the folder.</p>
            <div>
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
            </div>
            <Button onClick={() => refetch()}>Reload</Button>
        </div>}>
            <div>
                <p>Let's set things up. Select a TTS model to download.</p>
                <ul>
                    <For each={models()}>
                        {(item) => <div>{item.name} <button><IoPlay size={24} /></button></div>}
                    </For>
                </ul>
                <div>
                    <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Manually add TTS model</Button>
                    <Button className="p-2 bg-green-400" onClick={() => exitCompletedOnboarding()}>Done!</Button>
                </div>

            </div>
        </ErrorBoundary>
    )
}