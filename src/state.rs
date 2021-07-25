use crate::render::Vertex;

// size of the tiles in the spritesheet
const TW: f32 = 1. / 20.;
const TH: f32 = 1. / 3.;

/**
 * State of a tile on the game board
 * state: 0 = empty, 1 = cross, 2 = knot
 * frame is for keeping track of current animation frame
 */
#[derive(Copy, Clone)]
pub struct Tile {
  state: u8,
  frame: u8,
}

impl Tile {

  pub fn new() -> Self {
    return Tile {
      state: 0, frame: 0
    };
  }

}

/**
 * Keeps track of the game's state (turns, etc). Not involved with rendering.
 */
pub struct State {
  turn: bool, // true = X, false = O
  finished: bool
}

impl State {

  pub fn new() -> Self {
    return State { turn: true, finished: false };
  }

  /**
   * change the player in charge. also updates the game's title
   */
  pub fn change_turn(&mut self, window: &winit::window::Window) {
    self.turn = !self.turn;
    if self.turn {
      window.set_title("tic tac toe: cross turn");
    } else {
      window.set_title("tic tac toe: knots turn");
    }
  }

  /**
   * handle a click on a tile
   */
  pub fn handle_click(&mut self, mouse_pos: &winit::dpi::PhysicalPosition<f64>, board: &mut [[Tile; 3]; 3], window: &winit::window::Window) {

    // if the game has already been won, don't do anything
    if self.finished { return; }

    // get window size
    let size = window.inner_size();
    let tile_width = size.width as f64 / 3.;
    let tile_height = size.height as f64 / 3.;

    // determine which tile the mouse is clicking
    let x = (mouse_pos.x / tile_width) as usize;
    let y = (mouse_pos.y / tile_height) as usize;

    // if the tile is blank, place it then swap the turn!
    if board[x][y].state == 0 {
      // make a tile state that matches the current turn
      board[x][y] = Tile { frame: 0, state: match self.turn { true => 1, false => 2 } };

      // check for a victory
      self.check_victory(window, board);
      // if the game hasn't ended, change the term!
      if !self.finished {
        self.change_turn(window);
      }
    }

  }

  /**
   * check if any patterns for victory have been accomplished
   */
  pub fn check_victory(&mut self, window: &winit::window::Window, board: &[[Tile; 3]; 3]) {
    
    // column matches
    if ( board[0][0].state != 0 && board[0][0].state == board[0][1].state && board[0][1].state == board[0][2].state )
    || ( board[1][0].state != 0 && board[1][0].state == board[1][1].state && board[1][1].state == board[1][2].state )
    || ( board[2][0].state != 0 && board[2][0].state == board[2][1].state && board[2][1].state == board[2][2].state )
    // row matches
    || ( board[0][0].state != 0 && board[0][0].state == board[1][0].state && board[1][0].state == board[2][0].state )
    || ( board[0][1].state != 0 && board[0][1].state == board[1][1].state && board[1][1].state == board[2][1].state )
    || ( board[0][2].state != 0 && board[0][2].state == board[1][2].state && board[1][2].state == board[2][2].state )
    // diagonal matches
    || ( board[0][0].state != 0 && board[0][0].state == board[1][1].state && board[1][1].state == board[2][2].state )
    || ( board[2][0].state != 0 && board[2][0].state == board[1][1].state && board[1][1].state == board[0][2].state ) {

      // a victory!
      self.finished = true;
      window.set_title(match self.turn {
        true => "Congratulations Cross!",
        false => "Congratulations Knots!"
      });

    }

  }

}

/**
 * Create an empty game board
 */
pub fn gen_board() -> [[Tile; 3]; 3] {
  return [
    [ Tile::new(), Tile::new(), Tile::new() ],
    [ Tile::new(), Tile::new(), Tile::new() ],
    [ Tile::new(), Tile::new(), Tile::new() ]
  ];
}

pub fn gen_board_vertices(board: &mut [[Tile; 3]; 3]) -> Vec<Vertex> {

  // simplify things by breaking up the vertices before combining
  // simple two triangle square for the background, the texture is at the bottom of the sheet
  let mut background: Vec<Vertex> = vec![
    Vertex { pos: [ -1., 1. ], tex_coords: [ 0., TH * 2. ] },
    Vertex { pos: [ -1., -1. ], tex_coords: [ 0., 1. ] },
    Vertex { pos: [ 1., -1. ], tex_coords: [ TW, 1. ] },

    Vertex { pos: [ -1., 1. ], tex_coords: [ 0., TH * 2. ] },
    Vertex { pos: [ 1., -1. ], tex_coords: [ TW, TH * 3. ] },
    Vertex { pos: [ 1., 1. ], tex_coords: [ TW, TH * 2. ] },
  ];

  // create a list for the loop to write to
  let mut tiles: Vec<Vertex> = Vec::new();
  // loop through the tiles to generate vertices
  for (x, row) in board.clone().iter().enumerate() {
    // get each tile
    for (y, tile) in row.iter().enumerate() {
      // if the tile type is zero, no vertices needed
      if tile.state != 0 {
        // find the coordinates for the (top left of the) tile
        let tile_x: f32 = -1. + 0.666 * x as f32;
        let tile_y: f32 = 1. - 0.666 * y as f32;
        // get the coordinates for the texture to use
        let tex_x: f32 = TW * tile.frame as f32;
        let tex_y: f32 = TH * (tile.state - 1) as f32;

        // push the new vertices
        tiles.append(&mut vec![
          Vertex { pos: [ tile_x, tile_y ], tex_coords: [ tex_x, tex_y ] },
          Vertex { pos: [ tile_x, tile_y - 0.666 ], tex_coords: [ tex_x, tex_y + TH ] },
          Vertex { pos: [ tile_x + 0.666, tile_y - 0.666 ], tex_coords: [ tex_x + TW, tex_y + TH ] },

          Vertex { pos: [ tile_x, tile_y ], tex_coords: [ tex_x, tex_y ] },
          Vertex { pos: [ tile_x + 0.666, tile_y - 0.666 ], tex_coords: [ tex_x + TW, tex_y + TH ] },
          Vertex { pos: [ tile_x + 0.666, tile_y ], tex_coords: [ tex_x + TW, tex_y ] }
        ]);

        // if the animation frame isn't 19 (frame 20), update it for the next render
        if tile.frame < 19 {
          board[x][y].frame += 1;
        }
      }
    }
  }

  // make a shared list for all the vertices
  let mut vertices: Vec<Vertex> = Vec::new();
  vertices.append(&mut background);
  vertices.append(&mut tiles);

  return vertices;

}