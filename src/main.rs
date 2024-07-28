use imgui::Condition;
use imgui::ItemHoveredFlags;
use sdl2::video::SwapInterval;

use glow::HasContext;
use imgui::Context;
use imgui_glow_renderer::glow;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use rfd;
use sdl2::event::Event;

mod core;
use core::beep;
use core::{cpu::Cpu, screen};
use std::fs;
use std::time::Instant;

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::keyboard::Keycode;
use sdl2::AudioSubsystem;

mod graphics;

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

const SCALE: usize = 20;

const MENU_BAR_HEIGHT: usize = 39;

const WINDOW_WIDTH: usize = screen::WIDTH * SCALE;
const WINDOW_HEIGHT: usize = screen::HEIGHT * SCALE + MENU_BAR_HEIGHT;

fn main() {
    let mut loaded_rom = false;
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
        graphics::setup_opengl(&mut renderer)
    };

    // Setup Chip-8 and sound
    let mut cpu = Cpu::new();

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

    let mut running = true;
    'running_loop: while running {
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
                sdl2::event::Event::Quit { .. } => {
                    break 'running_loop;
                }
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
                if let Some(menu) = ui.begin_menu("File") {
                    let btn = ui
                        .menu_item_config("Load ROM")
                        .enabled(!cpu.rom_loaded)
                        .shortcut("Ctrl + O")
                        .build();
                    if btn {
                        rom_select_window(&mut cpu);
                    }
                    if ui
                        .menu_item_config("Close ROM")
                        .shortcut("Ctrl + W")
                        .enabled(cpu.rom_loaded)
                        .build()
                    {
                        cpu.clear();
                        unsafe {
                            graphics::update_render(
                                &mut renderer,
                                &mut buffer,
                                &tex,
                                &cpu.screen.0,
                            );
                        }
                    };
                    ui.separator();
                    ui.menu_item_config("Load state")
                        .shortcut("Ctrl + L")
                        .build();
                    ui.menu_item_config("Save state")
                        .shortcut("Ctrl + S")
                        .build();
                    ui.separator();
                    if ui.menu_item("Exit") {
                        running = false;
                    }
                    menu.end();
                }

                let inspect_enabled = cpu.is_halted() && cpu.rom_loaded;
                {
                    if let Some(_) = ui.begin_menu_with_enabled("Ispect", inspect_enabled) {
                        if let Some(_) = ui.begin_menu("Memory") {
                            ui.menu_item("View");
                            ui.menu_item("Edit");
                        }
                        if let Some(_) = ui.begin_menu("Registers") {
                            ui.menu_item("View");
                            ui.menu_item("Edit");
                        }
                        if let Some(_) = ui.begin_menu("Stack") {
                            ui.menu_item("View");
                            ui.menu_item("Edit");
                        }
                    }

                    if !inspect_enabled
                        && ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED)
                    {
                        if !cpu.rom_loaded {
                            ui.tooltip_text("Please load a ROM first.");
                        } else {
                            ui.tooltip_text("Please halt the emulation first.");
                        }
                    }
                }
                //disabled_scope.end();

                if let Some(menu) = ui.begin_menu("Options") {
                    ui.menu_item_config("Main options").enabled(false).build();
                    ui.menu_item("Timings");
                    ui.menu_item("Sound");
                    ui.menu_item("Key bindings");
                    ui.menu_item("Render");
                    ui.separator();
                    ui.menu_item_config("Advanced").enabled(false).build();
                    ui.menu_item("Quirks");
                    menu.end();
                }

                let halt_width = 55.0;
                let fps_width = 90.0;
                let margin =
                    ui.cursor_pos()[0] + ui.content_region_avail()[0] - halt_width - fps_width;
                let disabled_scope = ui.begin_disabled(false);
                {
                    ui.set_cursor_pos([margin, ui.cursor_pos()[1]]);
                    let text = if cpu.is_halted() { "Resume" } else { "Halt" };
                    if ui.button_with_size(text, [halt_width, 0.0]) {
                        cpu.toggle_halt();
                    }
                }

                disabled_scope.end();
                let margin = ui.cursor_pos()[0] + ui.content_region_avail()[0] - fps_width;
                let disabled_scope = ui.begin_disabled(true);
                {
                    ui.set_cursor_pos([margin, ui.cursor_pos()[1]]);
                    let align_scope =
                        ui.push_style_var(imgui::StyleVar::ButtonTextAlign([1.0, 0.0]));
                    {
                        let no_bg_scope =
                            ui.push_style_color(imgui::StyleColor::Button, [0.0, 0.0, 0.0, 0.0]);
                        {
                            let _ =
                                ui.button_with_size(format! {"FPS: {:.2}", fps}, [fps_width, 0.0]);
                        }
                        no_bg_scope.end();
                    }
                    align_scope.end();
                }
                disabled_scope.end();
            }
            main_menu.end();

            if !cpu.rom_loaded {
                let no_rom_msg = "No ROM loaded!";
                let text_size = ui.calc_text_size(no_rom_msg);
                ui.set_cursor_pos([
                    io.display_size[0] / 2.0 - text_size[0] / 2.0,
                    io.display_size[1] / 2.0 - text_size[1] / 2.0,
                ]);
                ui.text(no_rom_msg);
                let size = [80.0, 0.0];
                ui.set_cursor_pos([
                    io.display_size[0] / 2.0 - size[0] / 2.0,
                    io.display_size[1] / 2.0 + 15.0,
                ]);
                if ui.button_with_size("Load ROM", size) {
                    rom_select_window(&mut cpu);
                }
            }
        });

        let draw_data = imgui.render();

        unsafe {
            // Update buffer to the latest emulator screen
            // TODO: Do this only when necessary
            graphics::update_render(&mut renderer, &mut buffer, &tex, &cpu.screen.0);

            // Clear and draw the screen
            renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
            renderer
                .gl_context()
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            let _ = renderer.render(draw_data);

            window.gl_swap_window();
        }

        cpu.tick(200);

        // Although VSync is present, ensure we don't get more than 100fps
        timer_subsystem.delay(10); // 1000ms / 100fps = 10ms
    }
}

fn rom_select_window(cpu: &mut Cpu) {
    let path = std::env::current_dir().unwrap();
    let res = rfd::FileDialog::new()
        .add_filter("ch8", &["ch8"])
        .set_directory(&path)
        .pick_file();

    if let Some(file_path) = res {
        let rom = fs::read(file_path).unwrap();
        cpu.load_rom(rom, PROGRAM_BEGIN);
    }
}
