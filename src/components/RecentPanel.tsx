import { IoArrowDown } from "solid-icons/io"
import { RecentPanelButton } from "./RecentPanelButton"
import { createResource } from "solid-js"
import { commands } from "../bindings"

export const RecentPanel = () => {
    const recents = createResource(commands.getRecents)
    return (
        <div class='bg-white p-2 rounded shadow absolute bottom-0 left-1/2 -translate-x-1/2 max-w-[800px] gap-4 flex-col'>
            <div class=" flex flex-row font-bold gap-2 justify-center items-center p-2">< IoArrowDown />Recent</div>
                
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 border-box">
                {recents.map(recent => (
                    <RecentPanelButton />
                ))}
            <div class="border border-gray-400 min-w-40 min-h-20 p-2 text-gray-400 rounded">TTS Recordings will appear here when you do them</div>
            </div>
        </div>

    )
}

