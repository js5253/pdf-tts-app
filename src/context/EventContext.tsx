import { createContext, ParentProps, useContext } from "solid-js";
import { DownloadManagerProgress, HasCompletedOnboarding, TTSRequestProgress } from "../bindings";
import { Channel } from "@tauri-apps/api/core";
interface EventContextProps {
    onHasCompletedOnboarding: Channel<HasCompletedOnboarding>,
    onTtsGenerationProgressUpdate: Channel<TTSRequestProgress>,
    onModelDownloadProgressUpdate: Channel<DownloadManagerProgress>
}
const EventContext = createContext<EventContextProps>();
const EventContextProvider = (props: ParentProps) => {
    const onHasCompletedOnboarding = new Channel<HasCompletedOnboarding>();
    const onTtsGenerationProgressUpdate = new Channel<TTSRequestProgress>(); 
    const onModelDownloadProgressUpdate = new Channel<DownloadManagerProgress>(); 
    return <EventContext.Provider value={
        {
            onHasCompletedOnboarding,
            onTtsGenerationProgressUpdate,
            onModelDownloadProgressUpdate
        } as EventContextProps
    }>{props.children}</EventContext.Provider>
}
const useEventContext = () => {
    const { onHasCompletedOnboarding, onTtsGenerationProgressUpdate, onModelDownloadProgressUpdate } = useContext(EventContext);

    return { onHasCompletedOnboarding, onTtsGenerationProgressUpdate }
}
export { EventContext, EventContextProvider, useEventContext };
