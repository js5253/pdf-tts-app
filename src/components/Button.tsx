import { A, useLocation } from '@solidjs/router'
import { IoArrowBack, IoSettingsOutline } from 'solid-icons/io'
import { ParentProps } from 'solid-js'

type ButtonProps = {
    onClick: () => {},
    className?: string
}
export const Button = ({children, onClick, className}: ParentProps & ButtonProps) => {
    return <button class="p-2 bg-green-400 shadow rounded"  onClick={onClick}>
        {children}
    </button>
}