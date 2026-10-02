import { createResource } from "solid-js";

export const SHERPA_MODEL_ADDRESS = "https://k2-fsa.github.io/sherpa/onnx/tts/all/";

type SpectaResult<T> = {status: "ok", data: T} | {status: "error", error: string};

export function translateSpectaToSolidResource<T>(command: () => Promise<SpectaResult<T>>, options?: Parameters<typeof createResource<T>>[1]) {
    const getData = async () => {
        const result = await command();

        if (result.status === "error") {
            throw new Error(result.error + " - func " + command.name)
        }
        return result.data;
    }
    return createResource(getData, options)
}