import classNames from "classnames"

type ColorPickerInterface = {
    color: string,
    selected?: boolean
}

export const ColorPicker = ({color, selected = false}: ColorPickerInterface) => {
    console.log(color)
    return (
        <div class={classNames(`rounded-full w-12 h-12 bg-${color} outline outline-gray-200 hover:brightness-50`, selected && 'outline outline-2 outline-offset-3')}></div>

    )
}