import { A, useLocation } from '@solidjs/router'
import { IoArrowBack, IoSettingsOutline } from 'solid-icons/io'
export const Button = ({children, onClick, className}) => {
    return <button class="p-2 bg-green-400 shadow rounded"  onClick={onClick}>
        {children}
    </button>
}