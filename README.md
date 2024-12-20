# wgpu + Leptos

[![GitHub Pages](https://github.com/LioQing/wgpu-leptos-template/actions/workflows/github-pages.yml/badge.svg)](https://github.com/LioQing/wgpu-leptos-template/actions/workflows/github-pages.yml)

This project provides a Rust template for running [wgpu](https://wgpu.rs/) in a [Leptos](https://leptos.dev/) application.

This project is made possible by [WASM](https://webassembly.org/) and [WebGPU](https://www.w3.org/TR/webgpu/).

## Overview

Features:

- 🖼️ Provides a template for letting wgpu control a canvas in a Leptos application.
- 🤝 Enables interoperability between wgpu and Leptos through a pair of[`std::sync::mpsc::channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html).
- 🕹️ Demonstrates a real-time interactive example of a pyramid rendering.
- 🌐 Cross-platform compatibility enabled by wgpu to run both natively (only the canvas) and on the web.

Improvements:

- 📦 Be more modular and less ambiguous in the functionality and responsibility of each module.
- 💫 Fix the rubber banding when configurations are set by only sending signal with the changed field.
- 👾 Fix minor bugs related to controls, e.g. when window resizes.
- ⚙️ Add more configuration options.
