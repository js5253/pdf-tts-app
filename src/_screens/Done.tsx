import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir } from "@tauri-apps/api/path"
import { createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { IoPlay } from "solid-icons/io";

export const Done = () => {
    const navigate = useNavigate();
    const job = {
        name: "Job Name",
        segments: [{"fileName": "a.wav"}]
    }
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
        <h2>{job.name}</h2>
        {job.segments.map(segment => (
            <li>{segment.fileName} <button><IoPlay size={24} /></button></li>
        ))}
        <Button onClick={() => openPath(dataDir()!)}>View Output</Button>
        <Button onClick={navigateHome}>Make Another</Button>
    </>
}