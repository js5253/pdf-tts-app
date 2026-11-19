import { openPath } from "@tauri-apps/plugin-opener"
import { Button } from "../components/Button"
import { appDataDir } from "@tauri-apps/api/path"
import { createEffect, createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { IoPlay } from "solid-icons/io";
import { TTSCard } from "../components/TTSCard";
import { commands, TTSRequest } from "../bindings";
import { translateSpectaToSolidResource } from "../util";

export const JobProgress = () => {
    const [dataDir, setDataDir] = createSignal()
    const [inProgressJobs, setInProgressJobs] = createSignal<TTSRequest[]>();
    const [doneJobs, setDoneJobs] = createSignal<TTSRequest[]>();
    const [jobs] = translateSpectaToSolidResource(commands.getTtsProgress);
    createEffect(() => {
        jobs()?.forEach(job => {
            if (job.progress == "Completed") {
                setDoneJobs(items => (items ? [...items, job] : [job]));
            } else {
                setInProgressJobs(items => (items ? [...items, job] : [job]));

            }
        })
    }, [])
    onMount(async () => {
        console.log(await appDataDir())
        setDataDir(await appDataDir())
    })

    return <>
        <h1 class="text-4xl">Jobs</h1>
        <div class="w-full text-left">
            <h2 class="text-xl">In Progress</h2>
            <ul class="w-full">
                {inProgressJobs()?.map(item => <TTSCard item={item} />)}
            </ul>
            <h2 class="text-xl">Done</h2>
            <ul>
                {doneJobs()?.map(item => <TTSCard item={item} />)}
            </ul>
        </div>
    </>
}