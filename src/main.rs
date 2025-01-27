//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use lazy_static::lazy_static;
use std::{borrow::Cow, fs, path::PathBuf, time::Instant};

use glow::HasContext;
use imgui::{Condition, Context};
use imgui_glow_renderer::{glow, AutoRenderer};
use imgui_sdl2_support::SdlPlatform;
use mint::{Vector2, Vector3};
use rfd;

use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
    video::SwapInterval,
    AudioSubsystem,
};

mod core;
mod graphics;
use core::{beep, cpu::Cpu, screen};

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
    let mut loaded_rom_path: Option<PathBuf> = None;

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

    let mut active_palette_id = 0;
    let mut active_palette: ColorPalette = get_color_palette(active_palette_id).unwrap();

    let mut vsync_enabled = true;
    let mut max_fps: u32 = 200;

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
                        .enabled(!cpu.is_rom_loaded())
                        .shortcut("Ctrl + O")
                        .build();
                    if btn {
                        loaded_rom_path = rom_select_window(&mut cpu);
                    }
                    if ui
                        .menu_item_config("Restart ROM")
                        .shortcut("Ctrl + R")
                        .enabled(cpu.is_rom_loaded())
                        .build()
                    {
                        let rom = fs::read(loaded_rom_path.as_ref().unwrap()).unwrap();
                        cpu.clear();
                        cpu.load_rom(rom, PROGRAM_BEGIN);
                    };
                    if ui
                        .menu_item_config("Close ROM")
                        .shortcut("Ctrl + W")
                        .enabled(cpu.is_rom_loaded())
                        .build()
                    {
                        cpu.clear();
                        unsafe {
                            graphics::update_render(
                                &mut renderer,
                                &mut buffer,
                                &tex,
                                &cpu.screen.0,
                                &active_palette,
                            );
                        }
                    };
                    ui.separator();
                    if ui.menu_item("Exit") {
                        running = false;
                    }
                    menu.end();
                }

                if let Some(menu) = ui.begin_menu("Options") {
                    ui.menu_item_config("Main options").enabled(false).build();
                    if let Some(_) = ui.begin_menu("Timings & display") {
                        ui.text("Emulation and draw timings");
                        ui.slider("Draws per second", 30, 400, &mut cpu.draws_per_second);
                        ui.slider("Ticks/cycles per frame", 1, 500, &mut cpu.ticks_per_frame);

                        let cur_cursor = ui.cursor_pos();
                        ui.set_cursor_pos(Vector2 {
                            x: cur_cursor[0],
                            y: cur_cursor[1] + 20.0,
                        });
                        ui.separator();
                        ui.text("Display/window framerates");
                        if ui.checkbox(&"VSync", &mut vsync_enabled) {
                            if vsync_enabled {
                                let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);
                            } else {
                                let _ =
                                    video_subsystem.gl_set_swap_interval(SwapInterval::Immediate);
                            }
                        }
                        let disabled_region = ui.begin_disabled(vsync_enabled);
                        {
                            if ui.input_scalar("Max FPS", &mut max_fps).step(10).build() {
                                if max_fps < 10 {
                                    max_fps = 10
                                }
                            }
                        }
                        disabled_region.end();
                    }
                    if ui
                        .menu_item_config("Sound enabled")
                        .selected(cpu.is_beep_enabled())
                        .build()
                    {
                        cpu.toggle_beep_enabled();
                    }
                    if let Some(_) = ui.begin_menu("Color palette") {
                        if ui.combo(
                            "Active",
                            &mut active_palette_id,
                            COLOR_PALETTES.as_ref(),
                            |e| Cow::from(e.name),
                        ) {
                            if let Some(palette) = get_color_palette(active_palette_id) {
                                active_palette = palette;
                            } else {
                                active_palette.name = "Custom";
                            }
                        }

                        if active_palette.name == "Custom" {
                            ui.color_picker3("Enabled pixels", &mut active_palette.enabled_px);
                            ui.color_picker3("Disabled pixels", &mut active_palette.disabled_px);
                        }
                    }
                    ui.separator();
                    ui.menu_item_config("Advanced").enabled(false).build();

                    if let Some(_) = ui.begin_menu("Quirks") {
                        if ui
                            .menu_item_config("Shift operations against Vy instead of Vx register.")
                            .selected(cpu.shifts_against_vy)
                            .build()
                        {
                            cpu.shifts_against_vy = !cpu.shifts_against_vy
                        }

                        if ui
                            .menu_item_config(
                                "Memory load/save operations (fx55, fx65) increment I register.",
                            )
                            .selected(cpu.memory_load_save_increment_i)
                            .build()
                        {
                            cpu.memory_load_save_increment_i = !cpu.memory_load_save_increment_i
                        }

                        if ui
                            .menu_item_config("Sprite clipping instead of wrapping.")
                            .selected(cpu.sprite_clipping)
                            .build()
                        {
                            cpu.sprite_clipping = !cpu.sprite_clipping
                        }

                        if ui
                            .menu_item_config("Jump instructions to V0+NNN instead of VX+NN.")
                            .selected(cpu.jump_to_nnn)
                            .build()
                        {
                            cpu.jump_to_nnn = !cpu.jump_to_nnn
                        }
                    }

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

            if !cpu.is_rom_loaded() {
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
                    loaded_rom_path = rom_select_window(&mut cpu);
                }
            }
        });

        let draw_data = imgui.render();

        unsafe {
            // Update buffer to the latest emulator screen
            // TODO: Do this only when necessary
            graphics::update_render(
                &mut renderer,
                &mut buffer,
                &tex,
                &cpu.screen.0,
                &active_palette,
            );

            // Clear and draw the screen
            renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
            renderer
                .gl_context()
                .draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            let _ = renderer.render(draw_data);

            window.gl_swap_window();
        }

        cpu.tick();

        if !vsync_enabled {
            if max_fps < 1000 {
                timer_subsystem.delay(1000 / max_fps);
            }
        }
    }
}

fn rom_select_window(cpu: &mut Cpu) -> Option<PathBuf> {
    let path = std::env::current_dir().unwrap();
    let res = rfd::FileDialog::new()
        .add_filter("ch8", &["ch8"])
        .set_directory(&path)
        .pick_file();

    if let Some(file_path) = res {
        let rom = fs::read(file_path.clone()).unwrap();
        cpu.load_rom(rom, PROGRAM_BEGIN);
        return Some(file_path);
    }

    None
}

#[derive(Copy, Clone)]
struct ColorPalette {
    pub name: &'static str,
    pub enabled_px: Vector3<f32>,
    pub disabled_px: Vector3<f32>,
}

impl ColorPalette {
    pub fn new(name: &'static str, enabled: [u8; 3], disabled: [u8; 3]) -> ColorPalette {
        ColorPalette {
            name,
            enabled_px: Vector3::from_slice(&[
                enabled[0] as f32 / 255.0,
                enabled[1] as f32 / 255.0,
                enabled[2] as f32 / 255.0,
            ]),
            disabled_px: Vector3::from_slice(&[
                disabled[0] as f32 / 255.0,
                disabled[1] as f32 / 255.0,
                disabled[2] as f32 / 255.0,
            ]),
        }
    }
}

lazy_static! {
#[rustfmt::skip]
static ref COLOR_PALETTES: [ColorPalette; 5] = [
    ColorPalette::new("Default", [242, 251, 235], [23, 18, 25]),
    ColorPalette::new("Inverted", [23, 18, 25], [242, 251, 235]),
    ColorPalette::new("Brown", [253, 203, 85], [63, 41, 30]),
    ColorPalette::new("Red", [204, 14, 19], [43, 0, 0]),
    ColorPalette::new("Custom", [0, 0, 0], [0, 0, 0]), // TODO: Make custom saveable via a config
];
}

fn get_color_palette(idx: usize) -> Option<ColorPalette> {
    if let Some(palette) = COLOR_PALETTES.get(idx) {
        if palette.name != "Custom" {
            return Some(palette.clone());
        }
    }
    None
}
