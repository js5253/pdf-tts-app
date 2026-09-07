import { A, useLocation } from '@solidjs/router'
import { IoArrowBack, IoSettingsOutline } from 'solid-icons/io'
export const Button = ({children, onClick, className}) => {
    const location = useLocation();
    return <button class={className} onClick={onClick}>
        {children}
    </button>
}