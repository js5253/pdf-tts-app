import { IoSettingsOutline } from 'solid-icons/io'
export const AppBar = () => {
    return <nav class="backdrop-brightness-80 p-3 flex justify-end">
        <button><IoSettingsOutline size={24}/></button>
    </nav>
}