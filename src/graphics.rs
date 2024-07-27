use glow::HasContext;
use imgui_glow_renderer::glow;
use imgui_glow_renderer::AutoRenderer;

use super::screen;

const GL_VERTEX_TOP_MARGIN: f32 =
    (super::WINDOW_HEIGHT - super::MENU_BAR_HEIGHT) as f32 / super::WINDOW_HEIGHT as f32;

pub unsafe fn update_render(
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

pub unsafe fn setup_opengl(
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
