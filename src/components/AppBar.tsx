import { A, useLocation } from '@solidjs/router'
import { IoArrowBack, IoSettingsOutline } from 'solid-icons/io'
export const AppBar = () => {
    const location = useLocation();
    return <nav class="backdrop-brightness-80 p-3 shadow-md flex justify-between items-end">
        {location.pathname != "/" && <A href="/"><IoArrowBack size={24}/></A>}
        <A href="/settings"><IoSettingsOutline size={24}/></A>
    </nav>
}