import { Accessor, createSignal, JSX, onMount, ParentProps, Setter } from "solid-js";
import { convertFileSrc } from "@tauri-apps/api/core";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
// @ts-expect-error
import * as pdfjsLib from 'pdfjs-dist/build/pdf.mjs'
import { Button } from "../components/Button";
import { IoAdd, IoArrowBackOutline, IoArrowForwardOutline, IoArrowUpLeftBox, IoSquareOutline } from "solid-icons/io";
import { ImFolderMinus } from "solid-icons/im";
import { PDFPageProxy } from "pdfjs-dist";
import { RenderParameters } from "pdfjs-dist/types/src/display/api";
type PDFViewerAppProps = {
  pdfFilePath: string,
};

const MAX_CANVAS_WIDTH = 600;
const MAX_CANVAS_HEIGHT = 800;

type PdfPathProps = ParentProps & {
  pdfFilePath: string;
  fileRegion: Accessor<[[number, number], [number, number]] | null>;
  setFileRegion: Setter<[[number, number], [number, number]] | null>;

}
export const SelectRegion = ({ pdfFilePath, fileRegion, setFileRegion }: PdfPathProps) => {
  // here, add a component where we can pick the region
  // it can be as simple as using one page
  const [page, setPage] = createSignal<null | PDFPageProxy>(null);
  const [renderContext, setRenderContext] = createSignal<null | RenderParameters>(null);
  const [pageNumber, setPageNumber] = createSignal(1);
  const [pdfPageCount, setPdfPageCount] = createSignal(1);
  const [isRectangleSelectionOpen, setIsRectangleSelectionOpen] = createSignal(false);
  const [rectangleRangeEnd, setRectangleRangeEnd] = createSignal<[number, number] | null>(null);
  const [rectangleRangeStart, setRectangleRangeStart] = createSignal<[number, number] | null>(null);

  let canvas!: HTMLCanvasElement;
  async function renderPdf() {
    if (!canvas) throw new Error("Missing Canvas");


    var context = canvas.getContext("2d");

    //
    // The workerSrc property shall be specified.
    //
    pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

    // Opening PDF by passing its binary data as a string. It is still preferable
    // to use Uint8Array, but string or array-like structure will work too.
    var loadingTask = pdfjsLib.getDocument({ url: convertFileSrc(pdfFilePath) });
    var pdf = await loadingTask.promise;
    setPdfPageCount(1)
    // Fetch the first page.
    setPage(pdf.getPage(pageNumber));
    var scale = 1;
    var viewport = page()!.getViewport({ scale });
    // Support HiDPI-screens.
    var outputScale = (canvas.width / viewport.width) * scale;

    // Prepare canvas using PDF page dimensions.
    var context = canvas.getContext("2d");

    canvas.width = Math.floor(MAX_CANVAS_WIDTH * outputScale);
    canvas.height = Math.floor(MAX_CANVAS_HEIGHT * outputScale);
    canvas.style.width = Math.floor(MAX_CANVAS_WIDTH) + "px";
    canvas.style.height = Math.floor(MAX_CANVAS_HEIGHT) + "px";

    var transform = outputScale !== 1 ? [outputScale, 0, 0, outputScale, 0, 0] : null;

    // Render PDF page into canvas context.
    setRenderContext({
      canvasContext: context!,
      // @ts-expect-error
      transform,
      viewport,
    });
    page()?.render(renderContext()!);

  }
  const onMouseMove: JSX.EventHandler<HTMLCanvasElement, MouseEvent> = (e) => {
    let ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    page()?.render(renderContext()!);
    if (!rectangleRangeEnd() && !rectangleRangeStart()) {
      // render nothing
    } else if (rectangleRangeEnd()) {
      let x = rectangleRangeStart()![0];
      let y = rectangleRangeStart()![1];
      ctx.ellipse(x, y, 5, 5, 0, 0, 360, false);
      // render starting dot
    } else {
      // render range
      let x = rectangleRangeStart()![0];
      let y = rectangleRangeStart()![1];
      let width = rectangleRangeEnd()![0] - rectangleRangeStart()![0];
      let height = rectangleRangeEnd()![0] - rectangleRangeStart()![0];
      ctx.strokeRect(x, y, width, height);
    }
  }
  function onLeftPage() {
    setPageNumber(val => {
      if (val == 0) return 0;
      return val - 1
    })
  }
  function onRightPage() {
    setPageNumber(val => {
      if (val < pdfPageCount()) return pdfPageCount();
      return val - 1
    })
  }
  function onRectangleRangeClick() {
    setIsRectangleSelectionOpen(val => !val);
  }
  onMount(async () => {
    renderPdf()
  })
  const watchRectangleRegion: JSX.EventHandler<HTMLCanvasElement, MouseEvent> = (e) => {
    if (!isRectangleSelectionOpen()) return;

    if (rectangleRangeStart()) {
      setRectangleRangeEnd([e.offsetX, e.offsetY])
      setFileRegion([rectangleRangeStart()!, rectangleRangeEnd()!])
    } else {
      setRectangleRangeStart([e.offsetX, e.offsetY])

    }
  }

  return (
    <div class="bg-green-50 p-4 rounded space-y-4">
      <div class="p-4 bg-green-100 mx-auto text-center">

        <div class="flex flex-row justify-center gap gap-4"><Button title="Left Page" onClick={onLeftPage}><IoArrowBackOutline /></Button><Button title="Right Page" onClick={onRightPage}><IoArrowForwardOutline /></Button><Button title="Select Region" onClick={onRectangleRangeClick}><IoSquareOutline /></Button>
        </div>
      </div>

      <div class="flex-1 relative h-full flex items-center justify-center  flex-col">
        <canvas onMouseMove={onMouseMove} onClick={watchRectangleRegion} id="the-canvas" style="border: 1px solid black; direction: ltr;" width={600} height={800} ref={(el) => {
          canvas = el; // el is created but not yet added to the DOM
        }}></canvas>
      </div>
    </div>
  )
}