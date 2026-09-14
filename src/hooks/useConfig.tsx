import { createResource, onMount } from "solid-js";
import { commands, TtsAppConfig } from "../bindings";
import { listen } from "@tauri-apps/api/event";

export const useConfig = () => {
    const [initialConfig, {refetch, mutate}] = createResource<TtsAppConfig>(commands.getConfig);
    
    onMount(async () => {
        await listen('config-change', async () => {
            console.log("Updated Config!");
            await refetch();
        })
    })
    return [initialConfig(), refetch]
}