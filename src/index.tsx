/* @refresh reload */
import { render } from "solid-js/web";
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

            <div class="grow flex flex-col gap-4 items-center justify-center p-8">
                {props.children}
            </div>
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
