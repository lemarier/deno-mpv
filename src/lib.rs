#![windows_subsystem = "windows"]

// Deno
use deno_core::plugin_api::Buf;
use deno_core::plugin_api::Interface;
use deno_core::plugin_api::Op;
use deno_core::plugin_api::ZeroCopyBuf;
use serde::{Deserialize, Serialize};

use conrod_core::widget_ids;
use conrod_winit::{convert_event, WinitWindow};
use gfx::Device;
use glutin::GlContext;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

const WIN_H: u32 = 600;
const WIN_W: u32 = 1000;

type DepthFormat = gfx::format::DepthStencil;

// A wrapper around the winit window that allows us to implement the trait necessary for enabling
// the winit <-> conrod conversion functions.
struct WindowRef<'a>(&'a winit::Window);

#[no_mangle]
pub fn deno_plugin_init(interface: &mut dyn Interface) {
    interface.register_op("mpv_create_window", op_mpv_create_window);
}

// Implement the `WinitWindow` trait for `WindowRef` to allow for generating compatible conversion
// functions.
impl<'a> WinitWindow for WindowRef<'a> {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        winit::Window::get_inner_size(&self.0).map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        winit::Window::get_hidpi_factor(&self.0) as _
    }
}

unsafe extern "C" fn get_proc_address(arg: *mut c_void, name: *const c_char) -> *mut c_void {
    let arg: &glutin::GlWindow = &*(arg as *mut glutin::GlWindow);
    let name = CStr::from_ptr(name).to_str().unwrap();
    arg.get_proc_address(name) as *mut c_void
}

#[derive(Serialize)]
struct MPVNewResult {
    id: u32,
}

#[derive(Serialize)]
struct MPVResponse<T> {
    err: Option<String>,
    ok: Option<T>,
}

#[derive(Deserialize)]
struct MPVNewParams {
    url: String,
}

fn op_mpv_create_window(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    let mut response: MPVResponse<MPVNewResult> = MPVResponse {
        err: None,
        ok: None,
    };

    let params: MPVNewParams = serde_json::from_slice(data).unwrap();

    response.ok = Some(MPVNewResult { id: 1 });

    open_window(params.url);

    let result: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();

    Op::Sync(result)
}

fn open_window(video_file: String) {
    let builder = glutin::WindowBuilder::new()
        .with_title("CHOCHOTTE 101")
        .with_dimensions((WIN_W, WIN_H).into());

    let context = glutin::ContextBuilder::new()
        // tried GLES to enable vaapi egl, but it's not so simple since it requires a XCB extension
        // also seems to fail with gfx
        // but see https://github.com/jbg/conrod-android-skeleton
        //.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)));
        .with_multisampling(4);
    let mut events_loop = winit::EventsLoop::new();

    // Initialize gfx things
    let (mut window, mut device, mut factory, rtv, _) = gfx_window_glutin::init::<
        conrod_gfx::ColorFormat,
        DepthFormat,
    >(builder, context, &events_loop)
    .unwrap();
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    // Initialize mpv
    let ptr = &mut window as *mut glutin::GlWindow as *mut c_void;
    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
    mpv_builder
        .try_hardware_decoding()
        .expect("failed setting hwdec");
    mpv_builder
        .set_option("terminal", "yes")
        .expect("failed setting terminal");
    mpv_builder
        .set_option("msg-level", "all=v")
        .expect("failed setting msg-level");
    let mut mpv: Box<mpv::MpvHandlerWithGl> = mpv_builder
        .build_with_gl(Some(get_proc_address), ptr)
        .expect("Error while initializing MPV with opengl");
    mpv.command(&["loadfile", &video_file])
        .expect("Error loading file");

    let mut renderer =
        conrod_gfx::Renderer::new(&mut factory, &rtv, window.get_hidpi_factor() as f64).unwrap();

    // Create UI and Ids of widgets to instantiate
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();

    // load the font
    //ui.fonts
    //    .insert_from_file("./assets/fonts/NotoSans-Regular.ttf")
    //    .unwrap();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { canvas, list });

    let image_map = conrod_core::image::Map::new();

    'main: loop {
        let mut should_quit = false;
        events_loop.poll_events(|event| {
            // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
            if let Some(event) = convert_event(event.clone(), &WindowRef(window.window())) {
                ui.handle_event(event);
            }

            // Close window if the exit button is pressed
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    winit::WindowEvent::CloseRequested => should_quit = true,
                    winit::WindowEvent::Resized(logical_size) => {
                        let hidpi_factor = window.get_hidpi_factor();
                        let physical_size = logical_size.to_physical(hidpi_factor);
                        window.resize(physical_size);
                        let (new_color, _) = gfx_window_glutin::new_views::<
                            conrod_gfx::ColorFormat,
                            DepthFormat,
                        >(&window);
                        renderer.on_resize(new_color);
                    }
                    _ => {}
                }
            }
        });
        if should_quit {
            break 'main;
        }

        // If the window is closed, this will be None for one tick, so to avoid panicking with
        // unwrap, instead break the loop
        let (win_w, win_h): (u32, u32) = match window.get_inner_size() {
            Some(s) => s.into(),
            None => break 'main,
        };

        // Draw if anything has changed
        let mut needs_swap_buffers = false;
        let dpi_factor = window.get_hidpi_factor() as f32;
        let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

        // @TODO set mpv_is_playing to something reasonable
        let mpv_is_playing = true;
        if mpv_is_playing {
            mpv.draw(0, dims.0 as i32, -(dims.1 as i32))
                .expect("failed to draw on conrod window");
            needs_swap_buffers = true;
        }
        let maybe_primitives = if mpv_is_playing {
            Some(ui.draw())
        } else {
            ui.draw_if_changed()
        };
        if let Some(primitives) = maybe_primitives {
            //Clear the window
            // is this really needed?
            //renderer.clear(&mut encoder, CLEAR_COLOR);

            renderer.fill(
                &mut encoder,
                dims,
                dpi_factor as f64,
                primitives,
                &image_map,
            );

            renderer.draw(&mut factory, &mut encoder, &image_map);

            encoder.flush(&mut device);
            needs_swap_buffers = true;
        }
        if needs_swap_buffers {
            // see vsync at https://docs.rs/glium/0.19.0/glium/glutin/struct.GlAttributes.html
            window.swap_buffers().unwrap();
            device.cleanup();
        }
    }
}
