import { createResource, createSignal } from "solid-js";
import { commands } from "../bindings";
import { Button } from "../components/Button"
import { SHERPA_MODEL_ADDRESS, translateSpectaToSolidResource } from "../util";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir, BaseDirectory } from "@tauri-apps/api/path";
import { watch } from "@tauri-apps/plugin-fs";
import { TTSVoiceManager } from "../components/TTSVoiceManager";
import { ProgressBar } from "../components/ProgressBar";

export const Onboarding = () => {


    console.log("ONBOARD")
    return (
        <div class="space-y-4">
            <h1 class="text-4xl">Hey.</h1>
            <ProgressBar  progress={.6}/>
            <TTSVoiceManager />
            <hr />
            <div class="gap-2 flex flex-row">
            </div>

        </div>
    )
}