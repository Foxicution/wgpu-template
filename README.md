# WGPU (27.0.1) + Winit (0.30.12) Template

This is a template repository to get started with **[WGPU](https://wgpu.rs/)
(27.0.1)** + **[Winit](https://github.com/rust-windowing/winit) (0.30.12)**.

The template supports **cross-platform** compilation for **Windows**, **Linux**,
**MacOS** and **WebAssembly**, utilizing **WebGPU/WebGL** with
[trunk](https://trunkrs.dev/).

## Quickstart

```sh
# Clone the repository and open it
git clone https://github.com/Foxicution/wgpu-template
cd wgpu-template

# To run natively (Windows/MacOS/Linux)
cargo run

# To run on the web with WebAssembly (WASM)
# Add target wasm32-unknown-unknown for WASM builds
rustup target add wasm32-unknown-unknown

# Install trunk for WASM builds
cargo install --locked trunk

# With WebGL (for browsers that don't support WebGPU)
trunk serve --features webgl --open
# With WebGPU
trunk serve --open
```

For web builds, the app will be running on http://localhost:8080.

To check if your browser supports WebGPU go [here](https://webgpureport.org/).

# Contributing

If you run into any issues while using the template, feel free to submit a
[Github issue](https://github.com/Foxicution/wgpu-template/issues) or a
[Pull Request](https://github.com/Foxicution/wgpu-template).

# Resources

References and projects that helped create this template:

- [learn-wgpu](https://sotrh.github.io/learn-wgpu/)
- [wgpu_winit_example](https://github.com/w4ngzhen/wgpu_winit_example)
- [wgpu-0.20-winit-0.30-web-example](https://github.com/erer1243/wgpu-0.20-winit-0.30-web-example)
- [winit issue#3626](https://github.com/rust-windowing/winit/issues/3626)
- [shura](https://github.com/AndriBaal/shura)
- [wgpu-example](https://github.com/matthewjberger/wgpu-example)
- [wgpu_starter](https://github.com/N3xus8/wgpu_starter)
- [winit discussion#3667](https://github.com/rust-windowing/winit/discussions/3667)
