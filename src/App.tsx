import { createSignal } from "solid-js";
import { open } from '@tauri-apps/plugin-dialog';
import "./App.css";

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
    <main class="border border-8 border-green-400 h-screen bg-green-100 flex flex-col justify-center items-center gap-4">
      <h1 class="text-5xl">Drag your PDF</h1>
      <p>or</p>
      <button class="border border-green-700 p-2" on:click={() => greet().then()}>Select A File</button>
    </main>
  );
}

export default App;
