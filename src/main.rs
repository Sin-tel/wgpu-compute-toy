use std::error::Error;

mod bind;
mod blit;
mod context;
mod pp;
mod utils;
mod render;

fn main() -> Result<(), Box<dyn Error>> {
    return winit::main();
}

mod winit {
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use crate::context::init_wgpu;
    use crate::render::WgpuToyRenderer;
    use winit::event::{ElementState, Event, WindowEvent};

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ShaderMeta {
        uniforms: Vec<Uniform>,
        textures: Vec<Texture>,
        #[serde(default)]
        float32_enabled: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Uniform {
        name: String,
        value: f32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Texture {
        img: String,
    }

    async fn init() -> Result<WgpuToyRenderer, Box<dyn Error>> {
        let wgpu = init_wgpu(1280, 720, "").await?;
        let mut wgputoy = WgpuToyRenderer::new(wgpu);

        let filename = if std::env::args().len() > 1 {
            std::env::args().nth(1).unwrap()
        } else {
            // "examples/default.wgsl".to_string()
            "examples/davidar/buddhabrot.wgsl".to_string()
        };
        let shader = std::fs::read_to_string(&filename)?;


        if let Ok(json) = std::fs::read_to_string(std::format!("{filename}.json")) {
            let metadata: ShaderMeta = serde_json::from_str(&json)?;
            println!("{:?}", metadata);

            if !metadata.textures.is_empty() {
                panic!("texture from url not supported");
            }

            // for (i, texture) in metadata.textures.iter().enumerate() {
            //     let url = if texture.img.starts_with("http") {
            //         texture.img.clone()
            //     } else {
            //         std::format!("https://compute.toys/{}", texture.img)
            //     };
            //     let resp = client.get(&url).send().await?;
            //     let img = resp.bytes().await?.to_vec();
            //     if texture.img.ends_with(".hdr") {
            //         wgputoy.load_channel_hdr(i, &img)?;
            //     } else {
            //         wgputoy.load_channel(i, &img);
            //     }
            // }

            let uniform_names: Vec<String> =
                metadata.uniforms.iter().map(|u| u.name.clone()).collect();
            let uniform_values: Vec<f32> = metadata.uniforms.iter().map(|u| u.value).collect();
            if !uniform_names.is_empty() {
                wgputoy.set_custom_floats(uniform_names, uniform_values);
            }

            wgputoy.set_pass_f32(metadata.float32_enabled);
        }

        if let Some(source) = wgputoy.preprocess(&shader) {
            println!("{}", source.source);
            wgputoy.compile(source);
        }
        Ok(wgputoy)
    }

    pub fn main() -> Result<(), Box<dyn Error>> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let mut wgputoy = runtime.block_on(init())?;
        let screen_size = wgputoy.wgpu.window.inner_size();
        let start_time = std::time::Instant::now();
        let event_loop = std::mem::take(&mut wgputoy.wgpu.event_loop).unwrap();
        let device_clone = wgputoy.wgpu.device.clone();
        std::thread::spawn(move || loop {
            device_clone.poll(wgpu::Maintain::Wait);
        });

        let _ = event_loop.run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    wgputoy.set_mouse_pos(
                        position.x as f32 / screen_size.width as f32,
                        position.y as f32 / screen_size.height as f32,
                    );
                }
                WindowEvent::MouseInput { state, .. } => {
                    wgputoy.set_mouse_click(state == ElementState::Pressed);
                }
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        wgputoy.resize(size.width, size.height, 1.);
                    }
                }
                WindowEvent::RedrawRequested => {
                    let time = start_time.elapsed().as_micros() as f32 * 1e-6;
                    wgputoy.set_time_elapsed(time);
                    let future = wgputoy.render_async();
                    runtime.block_on(future);
                    wgputoy.wgpu.window.request_redraw();
                }
                _ => (),
            },
            _ => (),
        });
        Ok(())
    }
}
