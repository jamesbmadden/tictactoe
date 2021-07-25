use crate::state;

use std::borrow::Cow;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use std::convert::TryInto;

/**
 * a vertex for rendering
 */
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
  pub pos: [f32; 2],
  pub tex_coords: [f32; 2]
}

/**
 * Managing state for rendering
 */
pub struct Render {
  pub surface: wgpu::Surface,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub render_pipeline: wgpu::RenderPipeline,
  pub sc_desc: wgpu::SwapChainDescriptor,
  pub swap_chain: wgpu::SwapChain,
  pub swapchain_format: wgpu::TextureFormat,

  pub vertex_buf: wgpu::Buffer,
  pub index_count: u32,
  pub bind_group: wgpu::BindGroup
}

impl Render {

  pub async fn new(window: &winit::window::Window, board: &mut[[state::Tile; 3]; 3]) -> Self {

    // lets get going besties
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: Some(&surface)
    }).await.expect("Failed to find an appropriate adapter");
    // create device
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
      label: Some("Device"),
      features: wgpu::Features::empty(),
      limits: wgpu::Limits::default()
    }, None).await.expect("Failed to create device");

    // create the shader module
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
      flags: wgpu::ShaderFlags::VALIDATION
    });

    // load vertices
    let vertices = state::gen_board_vertices(board);
    let index_count = vertices.len() as u32;
    // create buffers
    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertices),
      usage: wgpu::BufferUsage::VERTEX
    });

    // load textures
    let tex_img_data = image::load_from_memory(include_bytes!("assets/spritesheet.png")).unwrap();
    let tex_img = tex_img_data.as_rgba8().unwrap();
    let tex_dimensions = tex_img.dimensions();

    let tex_size = wgpu::Extent3d {
      width: tex_dimensions.0,
      height: tex_dimensions.1,
      depth_or_array_layers: 1
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size: tex_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
      label: Some("Layers")
    });

    queue.write_texture(
      wgpu::ImageCopyTextureBase {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO
      },
      tex_img,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some((4 * tex_dimensions.0).try_into().unwrap()),
        rows_per_image: None
      },
      tex_size
    );

    // create texture view and sampler
    let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    // create bind group
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Bind Group Layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStage::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2
          },
          count: None
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStage::FRAGMENT,
          ty: wgpu::BindingType::Sampler {
            comparison: false,
            filtering: true
          },
          count: None
        }
      ]
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&tex_view)
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&tex_sampler)
        }
      ],
      label: Some("Bind Group")
    });

    // create render pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Pipeline Layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[]
    });
    let swapchain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
          step_mode: wgpu::InputStepMode::Vertex,
          attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2]
        }]
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[wgpu::ColorTargetState {
          format: swapchain_format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrite::ALL
        }]
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default()
    });

    // finally! lets make the swap chain :)
    let mut sc_desc = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
      format: swapchain_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    return Render {
      surface, device, queue, swapchain_format, render_pipeline, sc_desc, swap_chain,
      bind_group, vertex_buf, index_count
    };

  }

  /**
   * Take the current board state, write buffers
   */
  pub fn update(&mut self, board: &mut[[state::Tile; 3]; 3]) {

    let vertices = state::gen_board_vertices(board);
    self.index_count = vertices.len() as u32;

    // new buffer
    self.vertex_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertices),
      usage: wgpu::BufferUsage::VERTEX
    });

  }

  /**
   * Render the game :)
   */
  pub fn render(&self) {

    let frame = self.swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture").output;
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Encoder") });

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &frame.view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 1., g: 1., b: 1., a: 1.
            }),
            store: true
          }
        }],
        depth_stencil_attachment: None
      });

      rpass.set_pipeline(&self.render_pipeline);
      rpass.set_bind_group(0, &self.bind_group, &[]);
      rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
      rpass.draw(0..self.index_count, 0..1);
    }

    // finish rendering and free memory :)
    self.queue.submit(Some(encoder.finish()));

  }

}