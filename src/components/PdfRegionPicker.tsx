import { Component, createEffect, createSignal, onMount } from "solid-js";
import { usePDFSlick } from "@pdfslick/solid";
import { convertFileSrc } from "@tauri-apps/api/core";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
// @ts-expect-error
import * as pdfjsLib from 'pdfjs-dist/build/pdf.mjs'
import { Button } from "./Button";
import { IoAdd, IoArrowUpLeftBox, IoArrowUpRightBox, IoCrop, IoMailUnreadSharp } from "solid-icons/io";
import { ImFolderMinus } from "solid-icons/im";
type PDFViewerAppProps = {
  pdfFilePath: string,
};

const MAX_CANVAS_WIDTH = 400;
const MAX_CANVAS_HEIGHT = 400;

export const PDFRegionPicker: Component<PDFViewerAppProps> = ({ pdfFilePath }) => {
  
  let canvas!: HTMLCanvasElement;
  onMount(async () => {
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
    // Fetch the first page.
    var page = await pdf.getPage(1);
    var scale = 1.5;
    var viewport = page.getViewport({ scale });
    // Support HiDPI-screens.
    var outputScale = window.devicePixelRatio || 1;

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
  })

  return (
    <div class="">
      <p>Selected FIle: {pdfFilePath}</p>
      <div class="flex-1 relative h-full flex items-center justify-center  flex-col">
        <div><Button><IoAdd /></Button><Button><ImFolderMinus /></Button><Button><IoArrowUpLeftBox /></Button><Button><IoArrowUpRightBox/></Button> </div>
        <canvas id="the-canvas" style="border: 1px solid black; direction: ltr" ref={(el) => {
          canvas = el; // el is created but not yet added to the DOM
        }}></canvas>
      </div>
    </div>);
};

