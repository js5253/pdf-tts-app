import { PDFRegionPicker } from "../components/PdfRegionPicker"

export const SelectRegion = ({pdfFilePath}) => {
    return(
        // here, add a component where we can pick the region
        // it can be as simple as using one page

        <div><PDFRegionPicker pdfFilePath={pdfFilePath} /></div>
    )
}