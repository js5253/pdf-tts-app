/* @refresh reload */
import { ErrorBoundary, render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import { AppBar } from "./components/AppBar";
import "./App.css";
import { Home } from "./_screens/Home";
import { Settings } from "./_screens/Settings";
import { Done } from "./_screens/Done";
import { Toaster } from "solid-toast";
import { Onboarding } from "./_screens/Onboarding";
import { ParentProps, Suspense } from "solid-js";
import { commands, HasCompletedOnboarding } from "./bindings";
import { translateSpectaToSolidResource } from "./util";
import { Channel } from "@tauri-apps/api/core";
import { EventContextProvider, useEventContext } from "./context/EventContext";
import "@pdfslick/solid/dist/pdf_viewer.css";
import { FaSolidSpinner } from "solid-icons/fa";
import { SettingsContextProvider } from "./context/SettingsContext";
import { JobProgress } from "./_screens/JobProgress";
import { ModelDownloader } from "./_screens/ModelDownloader";

const Layout = (props: ParentProps) => {
    return (
        <>
            <AppBar />
            <ErrorBoundary fallback={(error, reset) => (
                <div class="h-screen flex flex-col justify-center items-center">
                    <h1 class="text-3xl italic">Achievement unlocked: how did we get here?</h1>
                    <h2 class="text-xl text-italic text-gray-500">(Error: {error})</h2>
                    <p>Try restarting the app.</p>
                </div>
            )}>
                <div class="grow flex flex-col gap-4 items-center justify-center p-8 pt-16">
                    {props.children}
                </div>
            </ErrorBoundary>
            <Toaster />

        </>
    );
};
const Wrapper = () => {
    return <Suspense fallback={<div class="bg-green-100 w-screen h-screen flex justify-center items-center"><FaSolidSpinner size={30} class="animate-spin" /></div>}><main class="h-screen bg-green-100 flex flex-col">
        <SettingsContextProvider>
            <EventContextProvider>
                <App />
            </EventContextProvider>
        </SettingsContextProvider>
    </main>
    </Suspense>
}

const App = () => {
    const [hasCompletedOnboarding, { refetch: refetchCompletedOnboarding }] = translateSpectaToSolidResource(commands.getCompletedOnboarding)
    const { onHasCompletedOnboarding } = useEventContext();
    onHasCompletedOnboarding.onmessage = () => {
        refetchCompletedOnboarding();
    }
    return (
        <Router root={Layout}>
            {hasCompletedOnboarding() ?
                <Route path="/" component={Home} /> :
                <Route path="/" component={Onboarding} />}
            <Route path="/model_downloads" component={ModelDownloader} />
            <Route path="/in_progress" component={JobProgress} />

            <Route path="/done" component={Done} />
            <Route path="/settings" component={Settings} />
        </Router>

    )
}
render(() => <Wrapper />, document.getElementById("root") as HTMLElement);
