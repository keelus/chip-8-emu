use imgui::Condition;
use sdl2::video::SwapInterval;

use glow::HasContext;
use imgui::Context;
use imgui_glow_renderer::glow;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::event::Event;

mod core;
use core::beep;
use core::{cpu::Cpu, screen};
use std::fs;
use std::time::Instant;

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::keyboard::Keycode;
use sdl2::AudioSubsystem;

// Sample SquareWave struct code from SDL2's example
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

// Sample AudioCallback impl code from SDL2's example
impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

struct BeepHandler {
    device: Option<AudioDevice<SquareWave>>,
    desired_spec: AudioSpecDesired,
    audio_subsystem: AudioSubsystem,
}

impl beep::BeepHandler for BeepHandler {
    fn start(&mut self) {
        if self.device.is_none() {
            let new_device = self
                .audio_subsystem
                .open_playback(None, &self.desired_spec, |spec| SquareWave {
                    phase_inc: 250.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.12,
                })
                .unwrap();

            new_device.resume();
            self.device = Some(new_device);
        }
    }

    fn stop(&mut self) {
        if self.device.is_some() {
            let cur_device = self.device.take().unwrap();
            cur_device.close_and_get_callback();
            self.device = None;
        }
    }
}

const PROGRAM_BEGIN: u16 = 0x0200;

const SCALE: usize = 10;

const MENU_BAR_HEIGHT: usize = 39;

const WINDOW_WIDTH: usize = screen::WIDTH * SCALE;
const WINDOW_HEIGHT: usize = screen::HEIGHT * SCALE + MENU_BAR_HEIGHT;

const GL_VERTEX_TOP_MARGIN: f32 = (WINDOW_HEIGHT - MENU_BAR_HEIGHT) as f32 / WINDOW_HEIGHT as f32;

fn main() {
    // Initialize SDL2 window
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let audio_subsystem = sdl.audio().unwrap();
    let mut timer_subsystem = sdl.timer().unwrap();
    let mut event_loop = sdl.event_pump().unwrap();

    let window = video_subsystem
        .window("Hello triangle!", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .opengl()
        .build()
        .unwrap();

    // Get GL context and setup screen
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);

    let _gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video_subsystem.gl_get_proc_address(s) as *const _)
    };
    let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);

    // Initialize Imgui
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut platform = SdlPlatform::init(&mut imgui);
    let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();

    // Get texture and buffer where the emulator will render
    let (mut buffer, tex) = unsafe {
        renderer.gl_context().clear_color(0.1, 0.1, 0.1, 1.0);
        setup_opengl(&mut renderer)
    };

    // Setup Chip-8 and sound
    let rom_name = "7-beep";
    let rom = fs::read(format!("./roms/{}.ch8", rom_name)).unwrap();

    let mut cpu = Cpu::new(rom, PROGRAM_BEGIN);

    let desired_spec = AudioSpecDesired {
        freq: Some(22_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    cpu.add_beep_handler(Box::new(BeepHandler {
        device: None,
        audio_subsystem,
        desired_spec,
    }));

    let mut last = Instant::now();

    'running: loop {
        let now = Instant::now();
        let diff = now.duration_since(last).as_secs_f64();
        let mut fps = 0.0;
        if diff != 0.0 {
            fps = 1.0 / diff;
        }
        last = now;

        for event in event_loop.poll_iter() {
            platform.handle_event(&mut imgui, &event);
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        match key {
                            Keycode::Num1 => cpu.keypad.set_key(1, false),
                            Keycode::Num2 => cpu.keypad.set_key(2, false),
                            Keycode::Num3 => cpu.keypad.set_key(3, false),
                            Keycode::Num4 => cpu.keypad.set_key(0xC, false),

                            Keycode::Q => cpu.keypad.set_key(4, false),
                            Keycode::W => cpu.keypad.set_key(5, false),
                            Keycode::E => cpu.keypad.set_key(6, false),
                            Keycode::R => cpu.keypad.set_key(0xD, false),

                            Keycode::A => cpu.keypad.set_key(7, false),
                            Keycode::S => cpu.keypad.set_key(8, false),
                            Keycode::D => cpu.keypad.set_key(9, false),
                            Keycode::F => cpu.keypad.set_key(0xE, false),

                            Keycode::Z => cpu.keypad.set_key(0xA, false),
                            Keycode::X => cpu.keypad.set_key(0, false),
                            Keycode::C => cpu.keypad.set_key(0xB, false),
                            Keycode::V => cpu.keypad.set_key(0xF, false),
                            _ => {}
                        }
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keycode {
                        match key {
                            Keycode::Num1 => cpu.keypad.set_key(1, true),
                            Keycode::Num2 => cpu.keypad.set_key(2, true),
                            Keycode::Num3 => cpu.keypad.set_key(3, true),
                            Keycode::Num4 => cpu.keypad.set_key(0xC, true),

                            Keycode::Q => cpu.keypad.set_key(4, true),
                            Keycode::W => cpu.keypad.set_key(5, true),
                            Keycode::E => cpu.keypad.set_key(6, true),
                            Keycode::R => cpu.keypad.set_key(0xD, true),

                            Keycode::A => cpu.keypad.set_key(7, true),
                            Keycode::S => cpu.keypad.set_key(8, true),
                            Keycode::D => cpu.keypad.set_key(9, true),
                            Keycode::F => cpu.keypad.set_key(0xE, true),

                            Keycode::Z => cpu.keypad.set_key(0xA, true),
                            Keycode::X => cpu.keypad.set_key(0, true),
                            Keycode::C => cpu.keypad.set_key(0xB, true),
                            Keycode::V => cpu.keypad.set_key(0xF, true),
                            _ => {}
                        }
                    }
                }

                _ => {}
            }
        }

        platform.prepare_frame(&mut imgui, &window, &event_loop);
        imgui.style_mut().window_rounding = 0.0;
        imgui.style_mut().window_border_size = 0.0;

        let ui = imgui.new_frame();
        let io = ui.io();
        let w = ui
            .window("A window!")
            .position([0.0, 0.0], Condition::Appearing)
            .size(
                [io.display_size[0], io.display_size[1]],
                Condition::Appearing,
            )
            .menu_bar(true)
            .draw_background(false)
            .movable(false)
            .no_decoration();

        w.build(|| {
            let main_menu = ui.begin_menu_bar().unwrap();
            {
                if let Some(menu) = ui.begin_menu("Menu1") {
                    ui.menu_item("Another option");
                    ui.menu_item("Another option");
                    menu.end();
                }
                let _ = ui.begin_menu(format! {"FPS: {:.2}", fps});
            }
            main_menu.end();
        });

        let draw_data = imgui.render();

        unsafe {
            // Update buffer to the latest emulator screen
            // TODO: Do this only when necessary
            update_render(&mut renderer, &mut buffer, &tex, &cpu.screen.0);

            // Clear and draw the screen
            renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
            renderer
                .gl_context()
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            let _ = renderer.render(draw_data);

            window.gl_swap_window();
        }

        cpu.tick(10);

        // Although VSync is present, ensure we don't get more than 100fps
        timer_subsystem.delay(10); // 1000ms / 100fps = 10ms
    }
}

unsafe fn update_render(
    renderer: &mut AutoRenderer,
    buffer: &mut [u8; screen::WIDTH * screen::HEIGHT * 3],
    texture: &glow::Texture,
    screen_data: &[u64; screen::HEIGHT],
) {
    // Update the buffer data
    let mut buff_idx = 0;
    for row in screen_data {
        for col in 0x0..screen::WIDTH {
            let mask: u64 = 0x1 << (63 - col);
            let pixel_on = (row & mask) != 0;

            if pixel_on {
                buffer[buff_idx] = 0xFF;
                buffer[buff_idx + 1] = 0xFF;
                buffer[buff_idx + 2] = 0xFF;
            } else {
                buffer[buff_idx] = 0x0;
                buffer[buff_idx + 1] = 0x0;
                buffer[buff_idx + 2] = 0x0;
            }

            buff_idx += 3;
        }
    }

    // Render the buffer into the texture
    renderer
        .gl_context()
        .bind_texture(glow::TEXTURE_2D, Some(*texture));
    renderer.gl_context().tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGB as i32,
        screen::WIDTH as i32,
        screen::HEIGHT as i32,
        0,
        glow::RGB,
        glow::UNSIGNED_BYTE,
        Some(buffer),
    );
}

unsafe fn setup_opengl(
    renderer: &mut AutoRenderer,
) -> ([u8; screen::WIDTH * screen::HEIGHT * 3], glow::Texture) {
    #[rustfmt::skip]
    let vertices: [f32; 16] = [
        1.0, GL_VERTEX_TOP_MARGIN, 1.0, 0.0, // Top-right
        -1.0, GL_VERTEX_TOP_MARGIN, 0.0, 0.0, // Top-left
        1.0, -1.0, 1.0, 1.0, // Bottom-right
        -1.0, -1.0, 0.0, 1.0, // Bottom-left
    ];
    let elements: [u32; 6] = [
        0, 1, 2, // Top-left triangle
        1, 2, 3, // Bottom-right triangle
    ];

    const VERTEX_SHADER_SRC: &str = "
        #version 150 core

        in vec2 position;
        in vec2 texcoord;
        out vec2 Texcoord;

        void main()
        {
            Texcoord = texcoord;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    ";

    const FRAGMENT_SHADER_SRC: &str = "
        #version 150 core

        in vec2 Texcoord;
        out vec4 outColor;
        uniform sampler2D tex;

        void main()
        {
            outColor = texture(tex, Texcoord);
        }
    ";

    // Setup vertex shader
    let vertex_shader = renderer
        .gl_context()
        .create_shader(glow::VERTEX_SHADER)
        .unwrap();
    renderer
        .gl_context()
        .shader_source(vertex_shader, VERTEX_SHADER_SRC);
    renderer.gl_context().compile_shader(vertex_shader);

    // Setup fragment shader
    let fragment_shader = renderer
        .gl_context()
        .create_shader(glow::FRAGMENT_SHADER)
        .unwrap();
    renderer
        .gl_context()
        .shader_source(fragment_shader, FRAGMENT_SHADER_SRC);
    renderer.gl_context().compile_shader(fragment_shader);

    // Combine vertex & fragment shaders into a program
    let shader_program = renderer.gl_context().create_program().unwrap();
    renderer
        .gl_context()
        .attach_shader(shader_program, vertex_shader);
    renderer
        .gl_context()
        .attach_shader(shader_program, fragment_shader);
    renderer
        .gl_context()
        .bind_frag_data_location(shader_program, 0, "outColor");
    renderer.gl_context().link_program(shader_program);
    renderer.gl_context().use_program(Some(shader_program));

    // VAO
    let vao = renderer.gl_context().create_vertex_array().unwrap();
    renderer.gl_context().bind_vertex_array(Some(vao));

    // Create the buffer to store the vertices
    let vbo = renderer.gl_context().create_buffer().unwrap();
    renderer
        .gl_context()
        .bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    renderer.gl_context().buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        &vertices.align_to::<u8>().1,
        glow::STATIC_DRAW,
    );

    // Create the element buffer to store both triangles
    let ebo = renderer.gl_context().create_buffer().unwrap();
    renderer
        .gl_context()
        .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
    renderer.gl_context().buffer_data_u8_slice(
        glow::ELEMENT_ARRAY_BUFFER,
        &elements.align_to::<u8>().1,
        glow::STATIC_DRAW,
    );

    // Setup shader variables
    let pos_attrib = renderer
        .gl_context()
        .get_attrib_location(shader_program, "position")
        .unwrap() as u32;
    renderer.gl_context().enable_vertex_attrib_array(pos_attrib);
    renderer.gl_context().vertex_attrib_pointer_f32(
        pos_attrib,
        2,
        glow::FLOAT,
        false,
        4 * std::mem::size_of::<f32>() as i32,
        0,
    );

    let tcoord_attrib = renderer
        .gl_context()
        .get_attrib_location(shader_program, "texcoord")
        .unwrap() as u32;
    renderer
        .gl_context()
        .enable_vertex_attrib_array(tcoord_attrib);
    renderer.gl_context().vertex_attrib_pointer_f32(
        tcoord_attrib,
        2,
        glow::FLOAT,
        false,
        4 * std::mem::size_of::<f32>() as i32,
        2 * std::mem::size_of::<f32>() as i32,
    );

    // Create the main texture for the emulator
    let tex = renderer.gl_context().create_texture().unwrap();
    renderer
        .gl_context()
        .bind_texture(glow::TEXTURE_2D, Some(tex));
    renderer
        .gl_context()
        .pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
    renderer.gl_context().tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    renderer.gl_context().tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::NEAREST as i32,
    );

    let buffer = [0 as u8; (screen::WIDTH * screen::HEIGHT * 3) as usize];
    renderer.gl_context().tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGB as i32,
        screen::WIDTH as i32,
        screen::HEIGHT as i32,
        0,
        glow::RGB,
        glow::UNSIGNED_BYTE,
        Some(&buffer),
    );

    (buffer, tex)
}
