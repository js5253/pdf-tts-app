# PDF-TTS-APP
Simple Tauri app that uses sherpa to create a text-to-speech of whatever pdf you send it. Made with Sherpa, Solid.js, and Tauri.

![project screenshot](resources/public/image.png)

## TTS Model Support
Currently, this project is hardcoded only to support models of the `vits-` caliber - they should have data like this:

- `vits-piper-en_US-libritts_r-medium`
    - `espeak-ng-data`
    - `en_US-libritts_r-medium.onnx`
    - `en_US-libritts_r-medium.onnx.json`
    - `MODEL_CARD`
    - `tokens.txt`

## Project Status
This project is under development. As of last edition of this README, the in-app downloading of TTS models is being worked on; the last update will be for simple PDF editing tools to exist (e.g, selecting a range of pages, potentially ignoring headers, better stripping of unnecessary characters).
    