import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir, dataDir } from "@tauri-apps/api/path"
import { createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";

export const Done = () => {
    const navigate = useNavigate();
    const [dataDir, setDataDir] = createSignal<string | null>(null);
    onMount(async () => {
        console.log(await appDataDir())
        setDataDir(await appDataDir())
    })
    const navigateHome = () => {
        navigate("/")
    }
    return <>
        <h1>Done!</h1>
        <Button onClick={() => openPath(dataDir())}>View Output</Button>
        <Button onClick={navigateHome}>Make Another</Button>
    </>
}