import { A, useLocation } from '@solidjs/router'
import { IoArrowBack, IoDownload, IoList, IoSettingsOutline } from 'solid-icons/io'
export const AppBar = () => {
    const location = useLocation();
    return <nav class="backdrop-brightness-80 p-3 shadow-md flex justify-between items-end fixed top-0 left-0 w-screen">
        {location.pathname != "/" && <A href="/"><IoArrowBack size={24} /></A>}
        <div class="flex flex-row gap-4 items-center">
            <A href="/model_downloads"><IoDownload /></A>
            <A href="/in_progress"><IoList /></A>
            {location.pathname != "/settings" && <A href="/settings"><IoSettingsOutline size={24} /></A>}
        </div>
    </nav>
}