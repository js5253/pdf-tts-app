import { createResource, createSignal } from "solid-js";
import { commands } from "../bindings";
import { Button } from "../components/Button"
import { SHERPA_MODEL_ADDRESS } from "../util";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { watch } from "@tauri-apps/plugin-fs";

export const Onboarding = () => {
    const [dataDir] = createResource<string>(appDataDir);
    // const models = createResource(await watch(
    //         'app.log',
    //         (event) => {
    //             console.log('app.log event', event);
    //         },
    //         {
    //             baseDir: BaseDirectory.AppData,
    //             delayMs: 500,
    //         }
    //     ));
    const exitCompletedOnboarding = async () => {
        await commands.setCompletedOnboarding(true);
    }

    console.log("ONBOARD")
    return (
        <div class="space-y-4">
            <h1 class="text-4xl">Hey.</h1>
            <p>Let's set things up. Download a TTS model from the following site, (VITS-only) and place it in the tts folder.</p>
            <hr />
            <p>Afterwards, select the TTS model from the list below.</p>
            <div class="gap-2 flex flex-row">
                <Button className="p-2 bg-blue-400" onClick={() => openUrl(SHERPA_MODEL_ADDRESS)}>Open Sherpa-ONNX models</Button>
                <Button className="p-2 bg-blue-400" onClick={() => openPath(dataDir() + "/tts")}>Open Directory</Button>
                <Button className="p-2 bg-blue-400" onClick={() => exitCompletedOnboarding()}>Close Screen</Button>
            </div>

        </div>
    )
}