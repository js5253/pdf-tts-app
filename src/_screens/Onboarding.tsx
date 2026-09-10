import { createResource, createSignal } from "solid-js";
import { commands } from "../bindings";
import { Button } from "../components/Button"
import { SHERPA_MODEL_ADDRESS } from "../util";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir } from "@tauri-apps/api/path";

export const Onboarding = () => {
    const [dataDir] = createResource<string>(appDataDir)

    console.log("ONBOARD")
    return (
        <div>
            <h1>Hey.</h1>
            <p>Let's set things up. Download a TTS model from the following site, (VITS-only) and place it in the tts folder.</p>
            <div class="gap-2 flex flex-row">
                <Button className="p-2 bg-blue-400" onClick={() => openUrl(SHERPA_MODEL_ADDRESS)}>Open Sherpa-ONNX models</Button>
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Open Directory</Button>
            </div>

        </div>
    )
}