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

      </main>
    </>
  );
}

export default App;
