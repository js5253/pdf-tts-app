import { A, useLocation } from '@solidjs/router'
import classNames from 'classnames'
import { IoArrowBack, IoSettingsOutline } from 'solid-icons/io'
import { ParentProps } from 'solid-js'

type ButtonProps = {
    onClick: () => void,
    className?: string
}
export const Button = ({children, onClick, className}: ParentProps & ButtonProps) => {
    return <button class={classNames("p-2 bg-green-400 shadow rounded hover:bg-green-500", className)}  onClick={onClick}>
        {children}
    </button>
}