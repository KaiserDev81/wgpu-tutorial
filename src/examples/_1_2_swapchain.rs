//## En este ejemplo creamos una clase State para controlar el renderizado en pantalla

use std::iter;

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        //## Our window needs to implement raw-window-handle's HasRawWindowHandle trait, 
        //## Fortunately, winit's Window fits the bill. We also need it to request our adapter

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };       //## The surface is used to create the swap_chain
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        //## The features field on DeviceDescriptor, allows us to specify what extra features we want. 
        //## For this simple example, I've deviced to not use any extra features.
        //## The device you have limits the features you can use. If you want to use certain features you may need to limit what devices you 
        //## support, or provide work arounds. You can get a list of features supported by your device using adapter.features(), or device.features().
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None, // Trace path
        ).await.unwrap();

        //## Here we are defining and creating the swap_chain. The usage field describes how the swap_chain's underlying textures will be used. 
        //## OUTPUT_ATTACHMENT specifies that the textures will be used to write to the screen (we'll talk about more TextureUsages later).
        //## The format defines how the swap_chains textures will be stored on the gpu. Usually you want to specify the format of the display you're using. 
        //## As of writing, I was unable to find a way to query what format the display has through wgpu, though there are plans on including such a method, 
        //## so wgpu::TextureFormat::Bgra8UnormSrgb will do for now. We use wgpu::TextureFormat::Bgra8UnormSrgb because that's the format that's guaranteed 
        //## to be natively supported by the swapchains of all the APIs/platforms which are currently supported.
        //## width and height, are self explanatory.
        //## The present_mode uses the wgpu::PresentMode enum which is defined as follows.
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,      // Inmediate, Mailbox, Fifo
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
        }
    }

    //## If we want to support resizing in our application, we're going to need to recreate the swap_chain everytime the window's size changes. That's the 
    //## reason we stored the physical size and the sc_desc used to create the swapchain. With all of these, the resize method is very simple.
    #[allow(unused_variables)]
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    //## input() returns a bool to indicate whether an event has been fully processed. If the method returns true, the main loop won't process the event any 
    //## further. We're just going to return false for now because we don't have any events we want to capture.
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) {
        //## Here's where the magic happens. First we need to get a frame to render to. This will include a wgpu::Texture and wgpu::TextureView that will hold 
        //## the actual image we're drawing to (we'll cover this more when we talk about textures).
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Timeout getting texture")
            .output;

        //## We also need to create a CommandEncoder to create the actual commands to send to the gpu. Most modern graphics frameworks expect commands to be 
        //## stored in a command buffer before being sent to the gpu. The encoder builds a command buffer that we can then send to the gpu.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        //## Now we can actually get to clearing the screen (long time coming). We need to use the encoder to create a RenderPass. The RenderPass has all the 
        //## methods to do the actual drawing. The code for creating a RenderPass is a bit nested, so I'll copy it all here, and talk about the pieces.
        {
            //## A RenderPassDescriptor only has two fields: color_attachments and depth_stencil_attachment. The color_attachements describe where we are going to draw our color to.
            //## We'll use depth_stencil_attachment later, but we'll set it to None for now.
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        //## We can't call encoder.finish() until we release that mutable borrow. The {} around encoder.begin_render_pass(...) tells rust to drop any variables 
        //## within them when the code leaves that scope thus releasing the mutable borrow on encoder and allowing us to finish() it.

        // submit will accept anything that implements IntoIter
        self.queue.submit(iter::once(encoder.finish()));
    }
}

pub fn main_1_2() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

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
                    // UPDATED!
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
                            // new_inner_size is &&mut so w have to dereference it twice
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