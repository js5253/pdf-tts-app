/* @refresh reload */
import { ErrorBoundary, render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import { AppBar } from "./components/AppBar";
import "./App.css";
import { Home } from "./_screens/Home";
import { Settings } from "./_screens/Settings";
import { Done } from "./_screens/Done";
import { Toaster } from "solid-toast";
const Layout = (props) => {
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
render(() =>
    <main class="h-screen bg-green-100 flex flex-col">
        <Router root={Layout}>
            <Route path="/" component={Home} />
            <Route path="/done" component={Done} />
            <Route path="/settings" component={Settings} />
        </Router>
    </main>, document.getElementById("root") as HTMLElement);
