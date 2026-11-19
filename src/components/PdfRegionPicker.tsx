import { Component, createSignal, onMount } from "solid-js";
import { convertFileSrc } from "@tauri-apps/api/core";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
// @ts-expect-error
import * as pdfjsLib from 'pdfjs-dist/build/pdf.mjs'
import { Button } from "./Button";
import { IoAdd, IoArrowBackOutline, IoArrowForwardOutline, IoArrowUpLeftBox, IoSquareOutline } from "solid-icons/io";
import { ImFolderMinus } from "solid-icons/im";
type PDFViewerAppProps = {
  pdfFilePath: string,
};

const MAX_CANVAS_WIDTH = 600;
const MAX_CANVAS_HEIGHT = 800;

export const PDFRegionPicker: Component<PDFViewerAppProps> = ({ pdfFilePath }) => {
  const [pageNumber, setPageNumber] = createSignal(0);
  const [pdfPageCount, setPdfPageCount] = createSignal(1);
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
    var page = await pdf.getPage(pageNumber);
    var scale = 1;
    var viewport = page.getViewport({ scale });
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
    var renderContext = {
      canvasContext: context,
      transform,
      viewport,
    };
    page.render(renderContext);

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
    })  }
  onMount(async () => {
    renderPdf()
  })

  return (
    <div class="bg-green-50 p-4 rounded space-y-4">
      <div class="p-4 bg-green-100 mx-auto text-center">

        <div class="flex flex-row justify-center gap gap-4"><Button title="Left Page" onClick={onLeftPage}><IoArrowBackOutline /></Button><Button title="Right Page" onClick={onRightPage}><IoArrowForwardOutline /></Button><Button title="Select Region"><IoSquareOutline /></Button>
        </div>
      </div>

      <div class="flex-1 relative h-full flex items-center justify-center  flex-col">
        <canvas id="the-canvas" style="border: 1px solid black; direction: ltr;" width={600} height={800} ref={(el) => {
          canvas = el; // el is created but not yet added to the DOM
        }}></canvas>
      </div>
    </div>);
};

