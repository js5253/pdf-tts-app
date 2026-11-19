import { onMount } from "solid-js";
import { useSettings } from "../context/SettingsContext";
import { Input } from "./Input";

export const AdvancedOptionsPanel = () => {
    const { config } = useSettings();
    console.log(config())
    return (
        <div class="backdrop-brightness-95 shadow w-[600px] rounded min-h-48 p-4 flex  flex-col mb-12">
            <h2 class="text-center font-bold text-xl">Advanced Options</h2>
            <form class="grid grid-cols-2 gap-2 items-center">
                <label for="combinePages">Combine pages into one audio file</label>
                <Input type="checkbox" name='combinePages' value={config().combine_pages} />
                <label for="whole-doc">Force OCR to get document text</label>
                <input type="checkbox" name="whole-doc" value={config().force_ocr} />
                <label for="outputPrefix">Output Prefix</label>
                <Input type="text" name='outputPrefix' value={config().output_prefix} />
                <label for="voice">Voice</label>
                <Input type="text" name='voice' value={config().voice} />
                <label for="speed">Speed</label>
                <Input type="number" name='speed' value={config().speed} />
                <label for="outputDir" class="">Output Directory</label>
                <Input type="number" name='speed' value={config().output_dir} />
            </form>
        </div>

    )
}