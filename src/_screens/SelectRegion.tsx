import { ParentProps } from "solid-js";
import { PDFRegionPicker } from "../components/PdfRegionPicker"
type PdfPathProps = ParentProps & {
    pdfFilePath: string;
}
export const SelectRegion = ({ pdfFilePath }: PdfPathProps) => {
    return (
        // here, add a component where we can pick the region
        // it can be as simple as using one page

        <div>
            {<p class="text-center text-xl">{pdfFilePath}</p>}
            {pdfFilePath.includes('.pdf') ? <PDFRegionPicker pdfFilePath={pdfFilePath} /> : <p>This file isn't a PDF so you can't edit it.</p>}
        </div>
    )
}