import { createSignal } from "solid-js";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import "./App.css";
import { AppBar } from "./components/AppBar";

const appWindow = getCurrentWindow();

await appWindow.onDragDropEvent((event) => {
  if (event.payload.type == 'over') {
    console.log(event.payload.position);
  } else if (event.payload.type === 'drop') {
    console.log('dropped:', event.payload.paths);
  } else if (event.payload.type === 'leave') {
    console.log('Drag cancelled.');
  }
});
function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const file = await open({
      multiple: false,
      directory: false,
    });
    console.log(file);
  }

  return (
    <>
      <main class="h-screen bg-green-100 flex flex-col">
        <AppBar />

        <div class="border border-8 border-green-400 grow flex flex-col gap-4 items-center justify-center">
          <h1 class="text-5xl">Drag your PDF</h1>
          <p>or</p>
          <button class="border border-green-700 p-2" on:click={() => greet().then()}>Select A File</button>
        </div>
      </main>
    </>
  );
}

export default App;
