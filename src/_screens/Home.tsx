import { Channel } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { createResource, createSignal, onCleanup, onMount, Suspense } from "solid-js"
import { useNavigate } from "@solidjs/router";
import toast from 'solid-toast';
import { commands, TtsAppConfig, TtsGenerationProgress } from '../bindings';
import { Button } from '../components/Button';
import { useConfig } from '../hooks/useConfig';
import { RecentPanel } from '../components/RecentPanel';
// naive attempt at using resources and solidjs better
const Home = () => {
    const [config] = useConfig();
    const navigate = useNavigate();
    const [isHovering, setIsHovering] = createSignal(false);
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
        const appWindow = getCurrentWindow();
        await appWindow.onDragDropEvent((event) => {
            if (event.payload.type == 'over') {
                setIsHovering(true);
            } else if (event.payload.type === 'drop') {
                setIsHovering(false);
                setCurrentFilePath(event.payload.paths[0]);
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
        console.log(config)
        if (!config || currentFilePath() === null) return;
        try {
            if (!currentFilePath()?.indexOf('.pdf') && !currentFilePath()?.indexOf('.epub') || !currentFilePath()?.indexOf('.docx')) throw new Error("invalid file type")

            await commands.runJob({
                ...config,
                input_file: currentFilePath()!,
                use_ocr: false
            }, onEvent);
        } catch (e) {
            toast("There was an error: " + (e as Error));
            console.log(e)
        }
    }

    return <Suspense fallback={<></>}>
        <>{currentFilePath() ? <>
            <p>Selected File: {currentFilePath()}</p>
            {/* <SelectRegion pdfFilePath={currentFilePath()} /> */}
            <div class="flex flex-row gap-4">
                <button class='border border-green-700 p-2' onClick={clearFile}>Back</button>
                <button class='border border-green-700 p-2 bg-black text-white' onClick={createTTS}>Continue</button>
            </div>
        </> : <div class='flex flex-col items-center justify-center outline outline-green-300 outline-10 outline-dashed p-4 gap-4 my-20'>
            {isHovering() && <div class='absolute bg-green-400/40 backdrop-blur-md w-screen h-screen top-0 left-0 flex items-center justify-center'>Drop me here!</div>}
            <h1 class="text-5xl text-center">Drag your file here</h1>
            <p>or</p>
            <Button className="bg-transparent border border-green-700 p-4 shadow-lg hover:backdrop-brightness-90 hover:text-white" onClick={openFilePicker}>Select A File</Button>
            <p class="text-gray-700 text-center">PDF, ePub, and DOCX are currently supported. Need only specific pages, regions, etc? Pre-process them in different software.</p>
        </div>}
        <RecentPanel />
        </>
    </Suspense>
}
export { Home };