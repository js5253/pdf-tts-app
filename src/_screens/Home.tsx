import { Channel, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { createSignal, onCleanup, onMount } from "solid-js"
import { useNavigate } from "@solidjs/router";
import toast from 'solid-toast';
import { commands, TtsAppConfig, TtsGenerationProgress } from '../bindings';
import { Button } from '../components/Button';


const Home = () => {
    const navigate = useNavigate();
    const [isHovering, setIsHovering] = createSignal(false);
    const [ttsConfig, setTtsConfig] = createSignal<TtsAppConfig>(); ///TODO
    const [currentFilePath, setCurrentFilePath] = createSignal<string | null>(null);
    const onEvent = new Channel<TtsGenerationProgress>(); // this is probably an erroneous line
    onEvent.onmessage = (message) => {
        if (message.event == "inProgress") {
            toast("Progress Update: " + (Number(message.data) * 100) + "%")
        } else {
            navigate("/done")
        }
    }
    onCleanup(() => {
    })
    onMount(async () => {
        const config = await commands.getConfig();
        if (config.status === "error") throw new Error();
        console.log(config);
        setTtsConfig(config.data);
        const appWindow = getCurrentWindow();
        await appWindow.onDragDropEvent((event) => {
            if (event.payload.type == 'over') {
                setIsHovering(true);
            } else if (event.payload.type === 'drop') {
                setIsHovering(false);
                setCurrentFilePath(event.payload.paths[0]);
                // console.log('dropped:', event.payload.paths);
            } else if (event.payload.type === 'leave') {
                setIsHovering(false);
            }
        });

    })
    function clearFile() {
        setCurrentFilePath(null)
    }
    async function openFilePicker() {
        const file = await open({
            multiple: false,
            directory: false,
        });
        setCurrentFilePath(file);
    }
    async function createTTS() {
        console.log(ttsConfig())
        const config = ttsConfig();
        if (!config || currentFilePath() === null) return;
        try {
            if (!currentFilePath()?.indexOf('.pdf') && !currentFilePath()?.indexOf('.epub') || !currentFilePath()?.indexOf('.docx')) throw new Error("invalid file type")

            await commands.runJob({
                ...config,
                input_file: currentFilePath()!,
                use_ocr: false
            }, onEvent);
            // navigate("/done")
        } catch (e) {
            toast("There was an error: " + (e as Error));
            console.log(e)
        }
    }

    return <>{currentFilePath() ? <>
        <p>Selected File: {currentFilePath()}</p>
        {/* <SelectRegion pdfFilePath={currentFilePath()} /> */}
        <div class="flex flex-row gap-4">
            <button class='border border-green-700 p-2' onClick={clearFile}>Back</button>
            <button class='border border-green-700 p-2 bg-black text-white' onClick={createTTS}>Continue</button>
        </div>
        <div class="backdrop-brightness-80 shadow w-[80vw] min-h-48 p-4">
            <h2>Advanced Options</h2>
            <form>
                <input type="checkbox" name='outputDir' />
                <label for="outputDir">Output Directory</label>
                <input type="text" name='voice' />
                <label for="voice">Voice</label>

                <input type="number" name='speed' />
                <label for="speed">Combine pages into one audio file</label>
                <input type="checkbox" name='outputDir' />
                <label for="outputDir">Output Directory</label>

            </form>
        </div>
    </> : <div class='flex flex-col items-center outline outline-green-300 outline-10 outline-dashed outline-offset-[100px] gap-4'>
        {/* {isHovering() && <div class='absolute bg-green-400 w-40 h-40'>Drop me here!</div>} */}
        <h1 class="text-5xl">Drag your file here</h1>
        <p>or</p>
        <Button className="bg-transparent border border-green-700 p-2 shadow-lg" onClick={() => openFilePicker().then()}>Select A File</Button>
        <p class="text-gray-700 text-center">PDF, ePub, and DOCX are currently supported. Need only specific pages, regions, etc? Pre-process them in different software.</p>
    </div>}
    </>
}
export { Home };