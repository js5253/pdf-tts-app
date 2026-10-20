import { createContext, ParentProps, useContext } from "solid-js";
import { commands, TtsAppConfig } from "../bindings";
import { translateSpectaToSolidResource } from "../util";
interface SettingsContextProps{
    config: TtsAppConfig
}
const SettingsContext = createContext<SettingsContextProps>();
const SettingsContextProvider = (props: ParentProps) => {
    const [config] = translateSpectaToSolidResource(commands.getConfig);
    const updateConfig = () => {

    }
    return <SettingsContext.Provider value={
        {config, updateConfig}
    }>{props.children}</SettingsContext.Provider>
}
const useSettings = () => {
    const { config } = useContext(SettingsContext);

    return { config }
}
export { SettingsContext, SettingsContextProvider, useSettings };
