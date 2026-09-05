const Settings = () => {
    return(
        <div class="flex-col">
            <h1>Default Options</h1>
            <input type="checkbox" name="sameOutputDirectory" />
            <label for="sameOutputDirectory">Put the output files in the same directory as the input file.</label>
            <input type="text" name="voice"/>
            <label for="voice">Voice</label>
            <input type="number" name="speed" />
            <label for="speed">Speed</label>
            <input type="checkbox" name="combinePages" />
            <label for="combinePages">Combine Pages</label>
            <input type="number" name="speakerId" />
            <label for="speakerId">Speaker ID</label>
        </div>
    )
}
export { Settings }