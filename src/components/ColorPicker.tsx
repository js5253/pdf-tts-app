import classNames from "classnames"

type ColorPickerInterface = {
    color: string,
    selected?: boolean,
    onClick: (color: string) => void
}

export const ColorPicker = ({color, selected = false, onClick}: ColorPickerInterface) => {
    console.log(color)
    return (
        <div class={classNames(`rounded-full w-12 h-12 bg-${color} outline outline-gray-200 outline-2 outline-offset-3 hover:brightness-50`, selected && 'outline-gray-200 ')}></div>

    )
}