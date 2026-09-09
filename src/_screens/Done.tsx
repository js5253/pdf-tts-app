import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir, dataDir } from "@tauri-apps/api/path"
import { createSignal, onMount } from "solid-js";

export const Done = () => {
        const [dataDir, setDataDir] = createSignal<string | null>(null);
        onMount(async () => {
        console.log(await appDataDir())
        setDataDir(await appDataDir())
        })
    return <>
    <h1>Done!</h1>
    <Button onClick={() => openPath(dataDir())}>View Output</Button>
    </>
}