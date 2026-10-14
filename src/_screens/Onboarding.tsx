import { createResource, createSignal, ErrorBoundary } from "solid-js";
import { commands } from "../bindings";
import { Button } from "../components/Button"
import { TTSVoiceManager } from "../components/TTSVoiceManager";
import { ProgressBar } from "../components/ProgressBar";
import { useEventContext } from "../context/EventContext";

export const Onboarding = () => {
    const {onHasCompletedOnboarding} = useEventContext();
    const onClick = async () => {
        await commands.setCompletedOnboarding(true, onHasCompletedOnboarding);
    }

    console.log("ONBOARD")
    return (
        <ErrorBoundary fallback={<p>Error   </p>}>
        <div class="space-y-4">
            <h1 class="text-4xl">Hey.</h1>
            <ProgressBar  progress={.6}/>
            <TTSVoiceManager />
            <hr />
            <div class="gap-2 flex flex-row">
                <Button onClick={onClick}>Next</Button>
            </div>

        </div>
        </ErrorBoundary>
    )
}