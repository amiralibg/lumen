//! wgpu renderer for lumen [`DrawData`]. One pipeline, one shader, one
//! atlas texture.
//!
//! The render target must be a non-sRGB format: lumen blends in gamma
//! space like browsers and design tools, so colors match the spec's hex
//! values exactly.

use lumen_core::paint::{DrawData, Vertex};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    screen: [f32; 2],
    scale: f32,
    _pad: f32,
}


pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforms: wgpu::Buffer,
    atlas: Option<(wgpu::Texture, u32)>,
    bind_group: Option<wgpu::BindGroup>,
    vbuf: Option<(wgpu::Buffer, u64)>,
    ibuf: Option<(wgpu::Buffer, u64)>,
    format: wgpu::TextureFormat,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lumen"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lumen"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lumen"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attrs = wgpu::vertex_attr_array![
            0 => Float32x2, 1 => Float32x2, 2 => Float32x2, 3 => Float32x2,
            4 => Float32x4, 5 => Float32, 6 => Uint32, 7 => Unorm8x4, 8 => Unorm8x4
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lumen"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                })],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lumen uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Renderer { pipeline, layout, uniforms, atlas: None, bind_group: None, vbuf: None, ibuf: None, format }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Upload buffers, uniforms and atlas changes for `data`.
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &DrawData) {
        let logical = [data.size_px[0] as f32 / data.scale, data.size_px[1] as f32 / data.scale];
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&Uniforms { screen: logical, scale: data.scale, _pad: 0.0 }));

        if let Some(d) = &data.atlas {
            let recreate = self.atlas.as_ref().is_none_or(|(_, s)| *s != d.atlas_size);
            if recreate {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("lumen atlas"),
                    size: wgpu::Extent3d { width: d.atlas_size, height: d.atlas_size, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                self.atlas = Some((tex, d.atlas_size));
                self.bind_group = None;
            }
            if d.w > 0 && d.h > 0 {
                let (tex, _) = self.atlas.as_ref().unwrap();
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: d.x, y: d.y, z: 0 }, aspect: wgpu::TextureAspect::All },
                    &d.data,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(d.w * 4), rows_per_image: Some(d.h) },
                    wgpu::Extent3d { width: d.w, height: d.h, depth_or_array_layers: 1 },
                );
            }
        }
        if self.atlas.is_none() {
            // No text yet: a 1x1 placeholder keeps the bind group valid.
            let tex = device.create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    label: Some("lumen atlas"),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &[0; 4],
            );
            self.atlas = Some((tex, 1));
        }
        if self.bind_group.is_none() {
            let view = self.atlas.as_ref().unwrap().0.create_view(&Default::default());
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lumen"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniforms.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                ],
            }));
        }

        // Safety: Vertex is repr(C), 64 bytes of plain f32/u32/u8 data with
        // no padding (asserted by a lumen-core test).
        let vbytes: &[u8] = unsafe {
            std::slice::from_raw_parts(data.vertices.as_ptr() as *const u8, data.vertices.len() * std::mem::size_of::<Vertex>())
        };
        let ibytes: &[u8] = bytemuck::cast_slice(&data.indices);
        upload(device, queue, &mut self.vbuf, vbytes, wgpu::BufferUsages::VERTEX);
        upload(device, queue, &mut self.ibuf, ibytes, wgpu::BufferUsages::INDEX);
    }

    /// Record draw calls into an open render pass.
    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>, data: &DrawData) {
        let (Some((vb, _)), Some((ib, _)), Some(bg)) = (&self.vbuf, &self.ibuf, &self.bind_group) else { return };
        if data.indices.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        for c in &data.cmds {
            let [x, y, w, h] = c.clip;
            if w == 0 || h == 0 {
                continue;
            }
            let x2 = (x + w).min(data.size_px[0]);
            let y2 = (y + h).min(data.size_px[1]);
            if x2 <= x || y2 <= y {
                continue;
            }
            pass.set_scissor_rect(x, y, x2 - x, y2 - y);
            pass.draw_indexed(c.index_start..c.index_start + c.index_count, 0, 0..1);
        }
    }

    /// Prepare and draw into `view`, clearing with the frame's clear color.
    pub fn draw_to(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, data: &DrawData) {
        self.prepare(device, queue, data);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("lumen") });
        {
            let c = data.clear;
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("lumen"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: c.r as f64, g: c.g as f64, b: c.b as f64, a: c.a as f64 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.render(&mut pass, data);
        }
        queue.submit([enc.finish()]);
    }
}

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, slot: &mut Option<(wgpu::Buffer, u64)>, bytes: &[u8], usage: wgpu::BufferUsages) {
    if bytes.is_empty() {
        return;
    }
    let need = bytes.len() as u64;
    if slot.as_ref().is_none_or(|(_, cap)| *cap < need) {
        let cap = need.next_power_of_two().max(4096);
        let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("lumen"), size: cap, usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        *slot = Some((buf, cap));
    }
    let (buf, _) = slot.as_ref().unwrap();
    // Pad to the copy alignment.
    let rem = bytes.len() % 4;
    if rem == 0 {
        queue.write_buffer(buf, 0, bytes);
    } else {
        let mut v = bytes.to_vec();
        v.resize(bytes.len() + 4 - rem, 0);
        queue.write_buffer(buf, 0, &v);
    }
}

/// Offscreen rendering for screenshots and tests. Native only (blocks).
#[cfg(not(target_arch = "wasm32"))]
pub mod offscreen {
    use super::*;

    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    /// Render `data` and read back tightly packed RGBA8 pixels.
    pub fn capture(device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut Renderer, data: &DrawData) -> Vec<u8> {
        let mut rb = Readback::default();
        rb.render(device, queue, renderer, data).to_vec()
    }

    /// An offscreen render target plus readback buffer, reused across
    /// frames. Hosts that composite lumen into another renderer (GPUI,
    /// a game engine's UI layer) render into this and upload the pixels.
    pub struct Readback {
        format: wgpu::TextureFormat,
        target: Option<(wgpu::Texture, wgpu::TextureView, wgpu::Buffer, [u32; 2])>,
        pixels: Vec<u8>,
    }

    impl Default for Readback {
        fn default() -> Self {
            Self::with_format(FORMAT)
        }
    }

    impl Readback {
        /// Read back in another 8-bit, 4-channel, non-sRGB format, such as
        /// `Bgra8Unorm` for hosts that upload BGRA. The [`Renderer`] passed
        /// to [`render`](Self::render) must be created with the same format.
        pub fn with_format(format: wgpu::TextureFormat) -> Self {
            Readback { format, target: None, pixels: Vec::new() }
        }

        /// Render `data` at `data.size_px` and return tightly packed,
        /// premultiplied pixels (RGBA8 unless made [`with_format`](Self::with_format)).
        /// Blocks until the GPU is done.
        pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut Renderer, data: &DrawData) -> &[u8] {
            let [w, h] = [data.size_px[0].max(1), data.size_px[1].max(1)];
            let row = (w * 4).div_ceil(256) * 256;
            if self.target.as_ref().is_none_or(|t| t.3 != [w, h]) {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("lumen offscreen"),
                    size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let view = tex.create_view(&Default::default());
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("lumen readback"),
                    size: (row * h) as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                self.target = Some((tex, view, buf, [w, h]));
            }
            let (tex, view, buf, _) = self.target.as_ref().unwrap();
            renderer.draw_to(device, queue, view, data);
            let mut enc = device.create_command_encoder(&Default::default());
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyBufferInfo { buffer: buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            queue.submit([enc.finish()]);
            let slice = buf.slice(..);
            slice.map_async(wgpu::MapMode::Read, |r| r.expect("map readback"));
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            {
                let mapped = slice.get_mapped_range().expect("mapped range");
                self.pixels.clear();
                self.pixels.reserve((w * h * 4) as usize);
                for y in 0..h {
                    let s = (y * row) as usize;
                    self.pixels.extend_from_slice(&mapped[s..s + (w * 4) as usize]);
                }
            }
            buf.unmap();
            &self.pixels
        }
    }

    /// Pipelined readback: render a frame, start copying it back, and pick
    /// the pixels up on a later frame instead of stalling for the GPU.
    /// Hosts show the newest finished frame meanwhile, one frame behind.
    pub struct AsyncReadback {
        format: wgpu::TextureFormat,
        target: Option<(wgpu::Texture, wgpu::TextureView, [u32; 2])>,
        slots: Vec<Slot>,
        seq: u64,
        /// Newest frame handed out; older frames that land later are stale.
        taken: u64,
        pixels: Vec<u8>,
    }

    struct Slot {
        buf: wgpu::Buffer,
        size: [u32; 2],
        row: u32,
        /// 0 free, 1 copying, 2 mapped, 3 failed.
        state: std::sync::Arc<std::sync::atomic::AtomicU8>,
        seq: u64,
    }

    /// A finished frame from [`AsyncReadback`].
    pub struct Frame<'a> {
        /// Tightly packed premultiplied pixels in the readback's format.
        pub pixels: &'a [u8],
        pub size: [u32; 2],
        /// Submission order: higher is newer.
        pub seq: u64,
    }

    const FREE: u8 = 0;
    const COPYING: u8 = 1;
    const MAPPED: u8 = 2;
    const FAILED: u8 = 3;

    impl AsyncReadback {
        /// `slots` frames may be in flight at once (2 or 3 is plenty). The
        /// [`Renderer`] must use `format`.
        pub fn new(format: wgpu::TextureFormat, slots: usize) -> Self {
            AsyncReadback { format, target: None, slots: Vec::with_capacity(slots.max(1)), seq: 0, taken: 0, pixels: vec![] }
        }

        fn capacity(&self) -> usize {
            self.slots.capacity().max(1)
        }

        /// Frames submitted and not yet collected.
        pub fn in_flight(&self) -> usize {
            use std::sync::atomic::Ordering;
            self.slots.iter().filter(|s| matches!(s.state.load(Ordering::Acquire), COPYING | MAPPED)).count()
        }

        /// Render `data` and start reading it back. Returns the frame's
        /// sequence number (see [`Frame::seq`]), or `None`, rendering nothing,
        /// when every slot is still busy; try again next frame.
        pub fn submit(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut Renderer, data: &DrawData) -> Option<u64> {
            use std::sync::atomic::{AtomicU8, Ordering};
            let [w, h] = [data.size_px[0].max(1), data.size_px[1].max(1)];
            let row = (w * 4).div_ceil(256) * 256;
            let free = self.slots.iter().position(|s| matches!(s.state.load(Ordering::Acquire), FREE | FAILED));
            let slot = match free {
                Some(i) => i,
                None if self.slots.len() < self.capacity() => {
                    self.slots.push(Slot {
                        buf: device.create_buffer(&wgpu::BufferDescriptor { label: Some("lumen readback"), size: 4, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false }),
                        size: [0, 0],
                        row: 0,
                        state: std::sync::Arc::new(AtomicU8::new(FREE)),
                        seq: 0,
                    });
                    self.slots.len() - 1
                }
                None => return None,
            };
            if self.target.as_ref().is_none_or(|t| t.2 != [w, h]) {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("lumen offscreen"),
                    size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let view = tex.create_view(&Default::default());
                self.target = Some((tex, view, [w, h]));
            }
            let s = &mut self.slots[slot];
            if s.size != [w, h] {
                s.buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("lumen readback"),
                    size: (row * h) as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                s.size = [w, h];
                s.row = row;
            }
            let (tex, view, _) = self.target.as_ref().unwrap();
            renderer.draw_to(device, queue, view, data);
            let mut enc = device.create_command_encoder(&Default::default());
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyBufferInfo { buffer: &s.buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            queue.submit([enc.finish()]);
            self.seq += 1;
            s.seq = self.seq;
            s.state.store(COPYING, Ordering::Release);
            let state = s.state.clone();
            s.buf.slice(..).map_async(wgpu::MapMode::Read, move |r| state.store(if r.is_ok() { MAPPED } else { FAILED }, Ordering::Release));
            Some(self.seq)
        }

        /// The newest finished frame, if one finished since the last call.
        /// Never blocks. Older finished frames are dropped.
        pub fn try_take(&mut self, device: &wgpu::Device) -> Option<Frame<'_>> {
            let _ = device.poll(wgpu::PollType::Poll);
            self.collect()
        }

        /// Wait for every frame in flight and return the newest.
        pub fn wait(&mut self, device: &wgpu::Device) -> Option<Frame<'_>> {
            if self.in_flight() > 0 {
                let _ = device.poll(wgpu::PollType::wait_indefinitely());
            }
            self.collect()
        }

        fn collect(&mut self) -> Option<Frame<'_>> {
            use std::sync::atomic::Ordering;
            for s in &self.slots {
                if s.seq <= self.taken && s.state.load(Ordering::Acquire) == MAPPED {
                    s.buf.unmap();
                    s.state.store(FREE, Ordering::Release);
                }
            }
            let newest = self.slots.iter().enumerate().filter(|(_, s)| s.state.load(Ordering::Acquire) == MAPPED).max_by_key(|(_, s)| s.seq).map(|(i, _)| i)?;
            // Pending older frames are stale once a newer one has landed.
            let newest_seq = self.slots[newest].seq;
            for (i, s) in self.slots.iter().enumerate() {
                if i != newest && s.seq < newest_seq && s.state.load(Ordering::Acquire) == MAPPED {
                    s.buf.unmap();
                    s.state.store(FREE, Ordering::Release);
                }
            }
            let s = &self.slots[newest];
            let [w, h] = s.size;
            {
                let mapped = s.buf.slice(..).get_mapped_range().expect("mapped range");
                self.pixels.clear();
                if s.row == w * 4 {
                    self.pixels.extend_from_slice(&mapped[..(w * h * 4) as usize]);
                } else {
                    self.pixels.reserve((w * h * 4) as usize);
                    for y in 0..h {
                        let o = (y * s.row) as usize;
                        self.pixels.extend_from_slice(&mapped[o..o + (w * 4) as usize]);
                    }
                }
            }
            s.buf.unmap();
            s.state.store(FREE, Ordering::Release);
            self.taken = s.seq;
            Some(Frame { pixels: &self.pixels, size: [w, h], seq: s.seq })
        }
    }

    /// Create a headless device.
    pub fn headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        // A real GPU first; then a software one (WARP on Windows, lavapipe
        // or llvmpipe on Linux), as on CI runners and VMs.
        let pick = |fallback: bool| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: fallback,
                ..Default::default()
            }))
            .ok()
        };
        let adapter = pick(false).or_else(|| pick(true))?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
    }
}
