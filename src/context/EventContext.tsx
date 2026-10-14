import { createContext, ParentProps, useContext } from "solid-js";
import { HasCompletedOnboarding, TtsGenerationProgress } from "../bindings";
import { Channel } from "@tauri-apps/api/core";
interface EventContextProps {
    onHasCompletedOnboarding: Channel<HasCompletedOnboarding>,
    onTtsGenerationProgressUpdate: Channel<TtsGenerationProgress>
}
const EventContext = createContext<EventContextProps>();
const EventContextProvider = (props: ParentProps) => {
    const onHasCompletedOnboarding = new Channel<HasCompletedOnboarding>(); // this is probably an erroneous line
    const onTtsGenerationProgressUpdate = new Channel<TtsGenerationProgress>(); // this is probably an erroneous line
    return <EventContext.Provider value={
        {
            onHasCompletedOnboarding,
            onTtsGenerationProgressUpdate
        } as EventContextProps
    }>{props.children}</EventContext.Provider>
}
const useEventContext = () => {
    const { onHasCompletedOnboarding, onTtsGenerationProgressUpdate } = useContext(EventContext);

    return { onHasCompletedOnboarding, onTtsGenerationProgressUpdate }
}
export { EventContext, EventContextProvider, useEventContext };
