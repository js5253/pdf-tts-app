import { onMount } from "solid-js";
import { useConfig } from "../hooks/useConfig"

export const AdvancedOptionsPanel = () => {
    const [config] = useConfig();

    return (
        <div class="backdrop-brightness-80 shadow w-[80vw] min-h-48 p-4">
            <h2>Advanced Options</h2>
            <form>
                <div>
                    <input type="checkbox" name='outputPrefix' value={config().output_prefix} />
                    <label for="outputPrefix">Output Prefix</label>
                </div>
                <div>
                    <input type="text" name='voice' value={config().voice} />
                    <label for="voice">Voice</label>
                </div>
                <div>
                    <input type="number" name='speed' value={config().speed} />
                    <label for="speed">Combine pages into one audio file</label>
                </div>
                <div>
                    <input type="checkbox" name='outputDir' value={config().output_dir} />
                    <label for="outputDir">Output Directory</label>
                </div>
            </form>
        </div>

    )
}