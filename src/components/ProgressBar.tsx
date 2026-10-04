interface ProgressBarProps {
    progress: number
}

export const ProgressBar = ({progress}: ProgressBarProps) => {
    return(
        <progress value={.5} class="rounded"></progress>
    )
}