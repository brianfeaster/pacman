use std::{
  fmt::Write,
  cmp::{max, min},
  collections::HashMap,
  io::{self, stdout, Stdout},
};

use crate::util::{Rnd, Term};

////////////////////////////////////////

// Pixels ᗣ ᗧ · ⋅ • ● ⬤
//pub const BLK :&str = "\x1b[3m "; // Inverted ASCII space
//pub const BLK :char = '█'; // full block 2588
pub const BLK  :char = '▉'; // left 7/8th block 2589
//pub const BLK :char = '◼'; // black medium square 25fc
//pub const BLK :char = '▮'; // black verticle rectangle 25ae
//pub const BLK :char = '▪'; // black small square 25aa
//pub const BLK :char = '🚽'; // black small square 25aa

////////////////////////////////////////

pub struct Dim {
    pub width: usize,
    pub height: usize,
    pub volume: usize,
}

const fn Dim(width: usize, height: usize) -> Dim {
    Dim{width, height, volume: width*height}
}

////////////////////

const MEMORYSIZE: usize = 256*256;
const SPRITECOUNT: usize = 16;

//// World Field / Window View(port) / Frame(buffer) Raster / CRT

pub const   TILE: Dim = Dim(8, 8);
pub const SPRITE: Dim = Dim(16, 16);

////////////////////

pub const WORLD :Dim = Dim(64, 64);
pub const FIELD :Dim = Dim(28, 33);

////////////////////

const   WINDOW: Dim = Dim(WORLD.width*TILE.width, WORLD.height*TILE.height);
pub const VIEW: Dim = Dim(FIELD.width*TILE.width, FIELD.height*TILE.height);

////////////////////

// TODO dynamic
pub const SPRITEFIELD: Dim = Dim(FIELD.width+SPRITE.width/TILE.width, FIELD.height+SPRITE.height/TILE.height);
pub const SPRITEVIEW:  Dim = Dim(SPRITEFIELD.width * TILE.width, SPRITEFIELD.height * TILE.height);

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone,Copy)]
pub struct Mloc {
  w: usize,
  h: usize,
  pub x: usize,
  pub y: usize,
}

impl Mloc {
  //pub const RT: usize = 0;
  //pub const DN: usize = 1;
  //pub const LF: usize = 2;
  //pub const UP: usize = 3;
  pub fn equal (&self, o: &Mloc) -> bool {
    self.x() == o.x() && self.y() == o.y()
  }
  pub fn new (w: usize, h: usize, x: usize, y: usize) -> Mloc { Mloc{w, h, x:(w<<32)+x, y:(h<<32)+y} }
  pub fn set (&mut self, x: usize, y: usize) { self.x=(self.w<<32)+x; self.y=(self.h<<32)+y }
  pub fn _area (&self) -> usize { self.w * self.h }
  pub fn shift (&mut self, dir: usize) -> &mut Mloc {
    match dir {
      0 => self.x += 1,
      1 => self.y += 1,
      2 => self.x -= 1,
      3 => self.y -= 1,
      _ => ()
    }
    self
  }
  pub fn _next (&self, dir: usize) -> Mloc {
    match dir {
      0 => Mloc{w:self.w, h:self.h, x:self.x+1, y:self.y},
      1 => Mloc{w:self.w, h:self.h, x:self.x,   y:self.y+1},
      2 => Mloc{w:self.w, h:self.h, x:self.x-1, y:self.y},
      3 => Mloc{w:self.w, h:self.h, x:self.x,   y:self.y-1},
      _ => *self
    }
  }
  pub fn x (&self) -> usize { self.x % self.w }
  pub fn y (&self) -> usize { self.y % self.h }
}

////////////////////////////////////////

#[derive(Clone,Copy,Default,PartialEq,Debug)]
pub struct Loc {
  pub x: usize,
  pub y: usize,
}
const fn Loc (x: usize, y: usize) -> Loc { Loc{x, y} }

#[derive(Clone,Copy)]
pub struct Sprite {
    pub data: usize,
    pub mrloc: Mloc,
    pub en: bool,
}

impl Default for Sprite {
  fn default () -> Sprite {
    Sprite{data:0, mrloc:Mloc{w:WINDOW.width, h:WINDOW.height, x:0, y:0}, en:false}
  }
}

pub struct Graphics {
    pub viewportSize: Loc,
    memory: [u8; MEMORYSIZE],
    memPtr: usize,
    c2b:    HashMap<char, u8>,
    field:   [u8; WORLD.volume],
    fieldDirty:  [bool; WORLD.volume],
    pub sprites: [Sprite; SPRITECOUNT],
    raster: [u8; WINDOW.volume],
    last:   [u8; WINDOW.volume],
    pub rnd: Rnd,
    pub msg: String,
    pub topleft: Loc,
    buff: String
}

impl Graphics {
  pub fn new(term: Term) -> Graphics {
    Graphics{
      viewportSize: Loc(min(term.w, VIEW.width), min(term.h, VIEW.height)),
      memory:  [0; MEMORYSIZE],
      memPtr:  0,
      c2b:     HashMap::<char, u8>::new(),
      field:     [1; WORLD.volume],
      fieldDirty:    [true; WORLD.volume],
      sprites:   [Sprite::default(); SPRITECOUNT],
      raster:   [0;  WINDOW.volume],
      last:     [255;WINDOW.volume],
      rnd:     Rnd::new(),
      msg:     String::new(),
      topleft: Loc::default(),
      buff: String::new()
    }
  }

  pub fn memorySetCharColorMap(&mut self, m2c: &[(char,u8)]) {
    self.c2b = m2c.iter().map(|e|*e).collect::<HashMap<char,u8>>()
  }

  pub fn initializeMemory(&mut self, s: &[&str]) -> usize {
    let start = self.memPtr;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.memory[self.memPtr] = self.c2b[&c];
            self.memPtr += 1;
        })
    });
    start
  }

  pub fn initializeFieldData(&mut self, m: &[(char,u8)], s: &[&str]) {
    let char2clr = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut y = 0;
    s.iter().for_each(|s| {
      s.chars().enumerate().for_each(|(x,c)|
        self.field[x+y*WORLD.width]=char2clr[&c] );
      y += 1;
    });
  }

  pub fn _getTile (&self, loc: Loc) -> u8 {
    self.field[loc.x + loc.y*WORLD.width]
  }

  fn getFieldTile (&self, x: usize, y: usize) -> u8 {
    self.field[x + y*WORLD.width]
  }

  pub fn getFieldTiles<F,R> (&self, xf: usize, yf: usize, f: F) -> [(R, usize, isize, isize); 4]
  where F: Fn(u8)->R
  {
    let y = yf*WORLD.width;
    [
      (f(self.field[(xf+1)%WORLD.width + y]),                      0, xf as isize + 1, yf as isize),
      (f(self.field[(xf+y+WORLD.width) % WORLD.volume]),            1, xf as isize,     yf as isize+1),
      (f(self.field[(xf+WORLD.width-1)%WORLD.width + y]),           2, xf as isize - 1, yf as isize),
      (f(self.field[(xf+y+WORLD.volume-WORLD.width) % WORLD.volume]),3, xf as isize,     yf as isize-1),
    ]
  }

  pub fn setFieldTile(&mut self, x: usize, y: usize, tile: u8) -> u8{
      let old = self.field[x+y*WORLD.width];
      self.field[x+y*WORLD.width] = tile;
      old
  }

  pub fn setSpriteLoc(&mut self, i: usize, x: usize, y:usize) {
    self.sprites[i].mrloc.x = x; // modulo WINDOW.width = 64 * 8
    self.sprites[i].mrloc.y = y;
  }
  pub fn setSpriteIdx(&mut self, i: usize, p: usize) {
    self.sprites[i].data = p;
  }
  pub fn rasterizeTilesSprites(&mut self, dataTiles: usize) {
    // tiles
    for fy in 0..FIELD.height {
    for fx in 0..FIELD.width {
      if !self.fieldDirty[fx+fy*WORLD.width] { continue }
      self.fieldDirty[fx+fy*WORLD.width]=false;
      let roffset = fy*TILE.volume*WORLD.width + fx*TILE.width;
      let mut ptr = self.getFieldTile(fx, fy) as usize * TILE.volume + dataTiles;
      for cy in 0..TILE.height {
      for cx in 0..TILE.width {
        self.raster [roffset + cx + cy*WINDOW.width] = self.memory [ptr];
        ptr += 1;
      }}
    }}

    // sprites
    for s in 0..SPRITECOUNT {
      if ! self.sprites[s].en { continue }

      let srlocy = self.sprites[s].mrloc.y();
      let srlocx = self.sprites[s].mrloc.x();

      // dirty tile bit
      let xsf = srlocx/TILE.width;
      let ysf = srlocy/TILE.height;
      //if WORLD.width  <= xsf { xsf=WORLD.width-1 }
      //if WORLD.height <= ysf { ysf=WORLD.height-1 }
      // TODO remove %WORLD.height
      self.fieldDirty[ xsf               + ysf%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+1)%SPRITEFIELD.width + ysf%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+2)%SPRITEFIELD.width + ysf%SPRITEFIELD.height*WORLD.width] = true;

      self.fieldDirty[ xsf               + (1+ysf)%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+1)%SPRITEFIELD.width + (1+ysf)%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+2)%SPRITEFIELD.width + (1+ysf)%SPRITEFIELD.height*WORLD.width] = true;

      self.fieldDirty[ xsf               + (2+ysf)%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+1)%SPRITEFIELD.width + (2+ysf)%SPRITEFIELD.height*WORLD.width] = true;
      self.fieldDirty[(xsf+2)%SPRITEFIELD.width + (2+ysf)%SPRITEFIELD.height*WORLD.width] = true;

      let spritedata = self.sprites[s].data;

      for cy in 0..SPRITE.height {
          let y = ((srlocy + cy) % WINDOW.height)% SPRITEVIEW.height;
          //if VIEW.height <= y { continue }
          for cx in 0..SPRITE.width {
              let x = ((srlocx + cx) % WINDOW.width)% SPRITEVIEW.width;
              //if VIEW.width <= x { continue }
              match self.memory[spritedata + cx+cy*SPRITE.width] {
                0 => continue, // transparent
                b => self.raster[x+y*WINDOW.width] = b
              } // match
          } // for x
      } // for y
    } // for s

  }

  pub fn alignViewport(&mut self, x: usize, y: usize) { // modulo SPRITEVIEWWIDTH = (28+2)*8 
    let w = self.viewportSize.x; // modulo Terminal or VIEW.width = 28*8
    let h = self.viewportSize.y;

    self.topleft = Loc(
      min(max(0, (x%VIEW.width + SPRITE.width/2-(w>>1))as isize), (VIEW.width-w)  as isize) as usize,
      min(max(0, (y%VIEW.height + SPRITE.height/2-(h>>1))as isize), (VIEW.height-h) as isize) as usize);
  }

  pub fn printField(&mut self) {
    let w = self.viewportSize.x;
    let h = self.viewportSize.y;

    self.buff.clear();
    write!(self.buff, "\x1b[H\x1b[30m").ok();

    let mut idx = 0;
    let mut ridx =  self.topleft.x + self.topleft.y*WINDOW.width;
    let mut lastColor=0;
    let mut loc=(0,0);

    for y in 0..h {
      for x in 0..w {
        let b = self.raster[ridx];
        let l = self.last[idx];
        if b!=l {
          self.last[idx] = b;
          // Update cursor
          if loc != (x, y) { write!(self.buff, "\x1b[{};{}H", y+1,x+1).ok(); }
          loc = (x+1, y);
          if 0==b {
            write!(self.buff, " ")
          } else {
            if lastColor == b {
              write!(self.buff, "{BLK}")
            } else {
              lastColor = b;
              write!(self.buff, "\x1b[38;5;{}m{BLK}", b)
            }
          }.ok();
        }
        ridx += 1;
        idx += 1;
      }
      idx += WINDOW.width-w;
      ridx += WINDOW.width-w;
    }
    <Stdout as io::Write>::write_all(&mut stdout(), self.buff.as_bytes()).ok();

    if !self.msg.is_empty() { // Maybe overlay debug msg
      //self.msg = format!("{:?}", locCenter);
      print!("\x1b[H\x1b[37m{}", self.msg);
      print!("\x1b[{};{}H\x1b[37m🚽", h/2, w/2);
      //self.msg.clear();
    }
  }
} // impl Graphics

////////////////////////////////////////
