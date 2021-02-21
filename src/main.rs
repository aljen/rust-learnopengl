extern crate nalgebra_glm as glm;

use gl::{types::*, VertexAttribPointer, STATIC_DRAW};
use glm::{U32Vec2, Vec3};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{event::Event, video};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

mod string_utils;
use string_utils::*;

mod resources;
use resources::*;

mod shader;
use shader::*;

mod program;
use program::*;

fn init_sdl() -> (sdl2::Sdl, sdl2::VideoSubsystem) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    (sdl_context, video_subsystem)
}

fn create_window(size: &U32Vec2, video: &sdl2::VideoSubsystem) -> sdl2::video::Window {
    let window = video
        .window("Window", size.x, size.y)
        .opengl()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    video.gl_set_swap_interval(0);

    window
}

fn create_opengl_context(
    window: &sdl2::video::Window,
    video: &sdl2::VideoSubsystem,
) -> sdl2::video::GLContext {
    let gl_attr = video.gl_attr();

    let gl_context = window.gl_create_context().unwrap();
    gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (4, 5));

    gl_context
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct Vertex {
    position: Vec3,
    color: Vec3,
}

impl Vertex {
    fn vertex_attrib_pointers() {
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as GLint,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as GLint,
                (3 * std::mem::size_of::<f32>()) as *const GLvoid,
            );
        }
    }
}

fn main() {
    let (sdl_context, video_subsystem) = init_sdl();

    let mut window_size = U32Vec2::new(900, 700);

    let window = create_window(&window_size, &video_subsystem);

    let _gl_context = create_opengl_context(&window, &video_subsystem);

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });

    let mut event_pump = sdl_context.event_pump().unwrap();

    unsafe {
        gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    let res = Resources::from_relative_exe_path(Path::new("../../assets")).unwrap();

    let shader_program = Program::from_res(&res, "shaders/triangle").unwrap();
    shader_program.mark_as_used();

    let mut last_frame = std::time::Instant::now();

    #[rustfmt::skip]
    let vertices: Vec<Vertex> = vec![
        Vertex{ position: Vec3::new(-0.5, -0.5, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
        Vertex{ position: Vec3::new( 0.5, -0.5, 0.0), color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex{ position: Vec3::new( 0.0,  0.5, 0.0), color: Vec3::new(0.0, 0.0, 1.0) },
    ];

    let mut vbo: GLuint = 0;

    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }

    println!(
        "len: {} size: {}",
        vertices.len(),
        std::mem::size_of::<Vec3>()
    );

    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    let mut vao: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }

    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        Vertex::vertex_attrib_pointers();

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    'running: loop {
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);

            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::SizeChanged(w, h) => {
                        window_size.x = w as u32;
                        window_size.y = h as u32;

                        unsafe {
                            gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        let now = std::time::Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui.io_mut().delta_time = delta_s;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        let ui = imgui.frame();
        ui.show_demo_window(&mut true);

        imgui_sdl2.prepare_render(&ui, &window);
        imgui_renderer.render(ui);

        window.gl_swap_window();
    }
}
