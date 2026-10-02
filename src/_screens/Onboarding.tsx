import { createResource, createSignal } from "solid-js";
import { commands } from "../bindings";
import { Button } from "../components/Button"
import { SHERPA_MODEL_ADDRESS, translateSpectaToSolidResource } from "../util";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { watch } from "@tauri-apps/plugin-fs";

export const Onboarding = () => {
    const [dataDir] = createResource<string>(appDataDir);
    const [models] = translateSpectaToSolidResource(commands.getOnlineModels);
    const exitCompletedOnboarding = async () => {
        await commands.setCompletedOnboarding(true);
    }

    console.log("ONBOARD")
    return (
        <div class="space-y-4">
            <h1 class="text-4xl">Hey.</h1>
            <p>Let's set things up. Select a TTS model to download.</p>
            {models()?.map(item => (
                <li>{item.name}</li>
            ))}
            <hr />
            <div class="gap-2 flex flex-row">
                <Button className="p-2 bg-blue-400" onClick={() => openUrl(SHERPA_MODEL_ADDRESS)}>Open Sherpa-ONNX models</Button>
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Open Directory</Button>
                <Button className="p-2 bg-blue-400" onClick={() => exitCompletedOnboarding()}>Close Screen</Button>
            </div>

        </div>
    )
}