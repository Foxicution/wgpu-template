use std::borrow::Cow;

use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Features, FragmentState, Instance, InstanceDescriptor, Limits, LoadOp, MemoryHints, Operations,
    PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp,
    Surface, SurfaceConfiguration, TextureFormat, TextureViewDescriptor, VertexState,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

pub async fn create_graphics(window: Rc<Window>, proxy: EventLoopProxy<Graphics>) {
    // Not `Instance::default()`. On the web, wgpu decides whether to use WebGPU when the
    // instance is created, and a plain `Instance::new` only checks that `navigator.gpu`
    // exists. Browsers can expose that object yet still fail to produce any WebGPU
    // adapter, in which case the instance is locked to WebGPU and never falls back to
    // WebGL. This helper probes for a real adapter first and drops BROWSER_WEBGPU if
    // there isn't one, so the `webgl` feature can actually take over.
    let instance = wgpu::util::new_instance_with_webgpu_detection(
        InstanceDescriptor::new_without_display_handle(),
    )
    .await;
    let surface = instance.create_surface(Rc::clone(&window)).unwrap();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(), // Power preference for the device
            force_fallback_adapter: false, // Indicates that only a fallback ("software") adapter can be used
            compatible_surface: Some(&surface), // Guarantee that the adapter can render to this surface
            apply_limit_buckets: false, // Rounds limits into coarse buckets to reduce fingerprinting. Only useful when exposing wgpu to untrusted content.
        })
        .await
        .expect("Could not get an adapter (GPU).");

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(), // Specifies the required features by the device request. Fails if the adapter can't provide them.
            required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints: MemoryHints::Performance,
            trace: Default::default(),
            experimental_features: Default::default(),
        })
        .await
        .expect("Failed to get device");

    // Get physical pixel dimensions inside the window
    let size = window.inner_size();
    // Make the dimensions at least size 1, otherwise wgpu would panic
    let width = size.width.max(1);
    let height = size.height.max(1);
    let mut surface_config = surface.get_default_config(&adapter, width, height).unwrap();

    // `get_default_config` picks the first present mode the surface reports, which
    // varies by platform and driver (often Mailbox, which renders uncapped). Pin Fifo
    // so the render loop is vsync-limited everywhere. Swap to Mailbox or Immediate for
    // uncapped frames.
    surface_config.present_mode = wgpu::PresentMode::Fifo;

    surface.configure(&device, &surface_config);

    let render_pipeline = create_pipeline(&device, surface_config.format);

    let gfx = Graphics {
        window: window.clone(),
        instance,
        surface,
        surface_config,
        adapter,
        device,
        queue,
        render_pipeline,
    };

    let _ = proxy.send_event(gfx);
}

fn create_pipeline(device: &Device, swap_chain_format: TextureFormat) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(swap_chain_format.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

#[derive(Debug)]
pub struct Graphics {
    window: Rc<Window>,
    instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl Graphics {
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {
        // `get_current_texture` reports why acquisition didn't yield a usable frame
        // rather than collapsing it into one error, so each case is handled on its own.
        let frame = match self.surface.get_current_texture() {
            // Suboptimal still presents correctly, it just no longer matches the surface.
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            // The surface configuration is stale. Reconfigure at the window's *current*
            // size — reusing the stored config would keep the same stale dimensions and
            // the surface would report Outdated forever, so no frame is ever acquired.
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                let size = self.window.inner_size();
                self.resize(size);
                return;
            }
            // Transient, skip this frame and try again on the next one.
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => return,
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r_pass.set_pipeline(&self.render_pipeline);
            r_pass.draw(0..3, 0..1);
        } // `r_pass` dropped here

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
    }
}
