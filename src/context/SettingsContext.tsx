import { createContext, ParentProps, useContext } from "solid-js";
import { commands, TtsAppConfig } from "../bindings";
import { translateSpectaToSolidResource } from "../util";
interface SettingsContextProps{
    config: TtsAppConfig,
    updateConfig: () => void
}
const SettingsContext = createContext<SettingsContextProps>();
const SettingsContextProvider = (props: ParentProps) => {
    const [config, {refetch: refetchConfig}] = translateSpectaToSolidResource(commands.getConfig);
    const updateConfig = async (newConfig: Partial<TtsAppConfig>) => {
        await commands.setConfig({
            ...config,
            ...newConfig
        })
        refetchConfig();
    }
    return <SettingsContext.Provider value={
        {config, updateConfig} as SettingsContextProps
    }>{props.children}</SettingsContext.Provider>
}
const useSettings = () => {
    const { config, updateConfig } = useContext(SettingsContext);

    return { config }
}
export { SettingsContext, SettingsContextProvider, useSettings };
