mod state;
mod render;

use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
  dpi::LogicalSize
};

async fn run() {

  // create a window
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().with_inner_size(LogicalSize::new(300, 300)).with_title("tic tac toe: cross turn").with_resizable(false).build(&event_loop).unwrap();

  // get the renderer and gameboard setup
  let mut board = state::gen_board();
  let mut state = state::State::new();
  let mut renderer = render::Render::new(&window, &mut board).await;
  
  // create a variable to keep track of mouse pos
  let mut mouse_pos: winit::dpi::PhysicalPosition<f64> = winit::dpi::PhysicalPosition::new(0., 0.);

  event_loop.run(move | event, _, control_flow | {

    match event {

      // rerender
      Event::RedrawRequested(_) => {
        renderer.update(&mut board);
        renderer.render();
      },

      // request redraw
      Event::MainEventsCleared => {
        window.request_redraw();
      },

      // close the window
      Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,

      // mouse has moved! Keep track
      Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => mouse_pos = position,

      // handle click!
      Event::WindowEvent { event: WindowEvent::MouseInput { state: winit::event::ElementState::Pressed, .. }, .. } => state.handle_click(&mouse_pos, &mut board, &window),

      _ => (),

    }

  });

}

fn main() {
  // TODO add web version for web compat :)
  futures::executor::block_on(run());
}
