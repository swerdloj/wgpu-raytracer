# WebGPU Ray Tracer

## About
Simple ray tracer written in Rust.

Based on [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

Implementations in the `shaders` directory exist for:
- GLSL compute shader
- GLSL fragment shader
- HLSL pixel (fragment) shader

I used this project as a means of learning HLSL, WebGPU, and the basics of ray tracing.

The project is a bit over-engineered as I also experimented with various interface designs regarding WebGPU, text rendering, windowing, and extensibility.

## WebGPU
WebGPU is a cross-platform graphics API similar to Vulkan. It can be run on platforms supporting OpenGL, DirectX11/12, Vulkan, Metal, and WebGPU itself (experimental in browsers).

wgpu-rs is the Rust implementation of WebGPU.