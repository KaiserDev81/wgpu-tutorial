use std::iter;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// Ejemplo de una estructura de un vertex para un buffer
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for Vertex {}     // Implementa el Buffer como PLAIN OLD DATA &[u8]
unsafe impl bytemuck::Zeroable for Vertex {}    // Implementa que se puede usar std::mem::zeroed()

// Implementacion de los Vertex
// 1. The stride defines how wide a vertex is. When the shader goes to read the next vertex, it will skip over stride number of bytes. In our case, stride 
//    will probably be 24 bytes.
// 2. step_mode tells the pipeline how often it should move to the next vertex. This seems redundant in our case, but we can specify 
//    wgpu::InputStepMode::Instance if we only want the change vertices when we start drawing a new instance. We'll cover instancing in a later tutorial.
// 3. Vertex attributes describe the individual parts of the vertex. Generally this is a 1:1 mapping with a structs fields which it is in our case.
// 4. This defines the offset in bytes that this attribute starts. The first attribute is usually zero, and any future attributes are the collective size_of 
//    the previous attributes data.
// 5. This tells the shader what location to store this attribute at. For example layout(location=0) in vec3 x in the vertex shader would correspond to the 
//    position field of the struct, while layout(location=1) in vec3 x would be the color field.
// 6. format tells the shader the shape of the attribute. Float3 corresponds to vec3 in shader code. The max value we can store in an attribute is 
//     Float4 (Uint4, and Int4 work as well). We'll keep this in mind for when we have to store things that are bigger than Float4.
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,   // 1.
            step_mode: wgpu::InputStepMode::Vertex,                         // 2.
            attributes: &[                                                  // 3.
                wgpu::VertexAttributeDescriptor {
                    offset: 0,                                              // 4.
                    shader_location: 0,                                     // 5.
                    format: wgpu::VertexFormat::Float3,                     // 6.
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

// Tambien se puede escribir con la macro vertex_attr_array
//impl Vertex {
//    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
//        wgpu::VertexBufferDescriptor {
//            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,   // 1.
//           step_mode: wgpu::InputStepMode::Vertex,                         // 2.
//            attributes: &wgpu::vertex_attr_array[0 => Float3, 1 => Float3],
//        }
//    }
//}

// La coleccion de Vertex a representar son VERTICES
// We arrange the vertices in counter clockwise order: top, bottom left, bottom right. We do it this way partially out of tradition, but mostly because we 
// specified in the rasterization_state of the render_pipeline that we want the front_face of our triangle to be wgpu::FrontFace::Ccw so that we cull the 
// back face. This means that any triangle that should be facing us should have its vertices in counter clockwise order.
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    // NEW!
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module = device.create_shader_module(wgpu::include_spirv!("shaders/shader_1_4.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shaders/shader_1_4.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            // NUEVO, es diferente del anterior vertex_stage: wgpu::ProgrammableStageDescriptor
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // Creacion del buffer de vertices
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        // Creacion del buffer de indices
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            size,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Timeout getting texture")
            .output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            // 1. The method name is set_index_buffer not set_index_buffers. You can only have one index buffer set at a time.
            // 2. When using an index buffer, you need to use draw_indexed. The draw method ignores the index buffer. Also make sure you use the number of 
            //    indices (num_indices), not vertices as you model will either draw wrong, or the method will panic because there are not enough indices.
            render_pass.set_pipeline(&self.render_pipeline);
            // NUEVO, pasamos el indice del Vertex buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // NUEVO, pasamos el indice del Index Buffer
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            // render_pass.draw(0..self.num_vertices, 0..1);        // Cuando no usamos el Index Buffer
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
    }
}

pub fn main_1_4() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}