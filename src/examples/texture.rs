use anyhow::*;
use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3d, we represent our 2d texture
            // by setting depth to 1.
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // SAMPLED tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            // The actual pixel data
            rgba,
            // The layout of the texture
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            size,
        );

        // Now that our texture has data in it, we need a way to use it. This is where a TextureView and a Sampler come in. A TextureView offers us a view 
        // into our texture. A Sampler controls how the Texture is sampled. Sampling works similar to the eyedropper tool in Gimp/Photoshop. Our program 
        // supplies a coordinate on the texture (known as a texture coordinate), and the sampler then returns a color back based on it's internal parameters.
        
        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // The address_mode_* parameter's determine what to do if the sampler get's a texture coordinate that's outside of the texture. There's a few that we 
        // can use.
        // ClampToEdge: Any texture coordinates outside the texture will return the color of the nearest pixel on the edges of the texture.
        // Repeat: The texture will repeat as texture coordinates exceed the textures dimensions.
        // MirrorRepeat: Similar to Repeat, but the image will flip when going over boundaries.

        // The mag_filter and min_filter options describe what to do when a fragment covers multiple pixels, or there are multiple fragments for one pixel respectively. This often comes into play when viewing a surface from up close, or far away. There are 2 options:
        // Linear: This option will attempt to blend the in-between fragments so that they seem to flow together.
        // Nearest: In-between fragments will use the color of the nearest pixel. This creates an image that's crisper from far away, but pixelated when 
        // view from close up. This can be desirable however if your textures are designed to be pixelated such is in pixel art games, or voxel games like 
        // Minecraft.

        // Mimmap_filter tiene unos parametros similares a min mag filters (son parecidos a OpenGL)

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}