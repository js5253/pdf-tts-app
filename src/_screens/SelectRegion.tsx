import { ParentProps } from "solid-js";
import { PDFRegionPicker } from "../components/PdfRegionPicker"
type PdfPathProps = ParentProps & {
    pdfFilePath: string;
}
export const SelectRegion = ({pdfFilePath}: PdfPathProps) => {
    return(
        // here, add a component where we can pick the region
        // it can be as simple as using one page

        <div><PDFRegionPicker pdfFilePath={pdfFilePath} /></div>
    )
}