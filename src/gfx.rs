use std::{
  fmt::Write,
  cmp::{max, min},
  collections::HashMap,
  io::{self, stdout, Stdout},
};

use crate::util::{Rnd, Term};

////////////////////////////////////////

// Pixels ᗣ ᗧ · ⋅ • ● ⬤
//pub const BLK: &str = "\x1b[3m "; // Inverted ASCII space
//pub const BLK: char = '█'; // full block 2588
pub const BLK:  char = '▉'; // left 7/8th block 2589
//pub const BLK: char = '◼'; // black medium square 25fc
//pub const BLK: char = '▮'; // black verticle rectangle 25ae
//pub const BLK: char = '▪'; // black small square 25aa
//pub const BLK: char = '🚽'; // black small square 25aa

////////////////////////////////////////

#[derive(Debug)]
pub struct Dim {
    pub width: usize,
    pub height: usize,
    pub volume: usize,
}

const fn Dim(width: usize, height: usize) -> Dim {
  Dim{width, height, volume: width*height}
}

////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Mvec {
  pub w: usize,
  pub h: usize,
  pub x: usize,
  pub y: usize,
}

impl Mvec {
  pub fn equal (&self, o: &Mvec) -> bool {
    self.x() == o.x() && self.y() == o.y()
  }
  pub fn new (w: usize, h: usize, x: usize, y: usize) -> Mvec {
    Mvec{w, h,
      x: (w<<32) + x,
      y: (h<<32) + y
    }
  }
  pub fn set (&mut self, x: usize, y: usize) {
    self.x = x + (self.w << 32);
    self.y = y + (self.h << 32);
  }
  pub fn shift (&mut self, dir: usize, mag: usize) {
    match dir {
      0 => self.x += mag,
      1 => self.y += mag,
      2 => self.x -= mag,
      3 => self.y -= mag,
      _ => ()
    }
  }
  pub fn x (&self) -> usize { self.x % self.w }
  pub fn y (&self) -> usize { self.y % self.h }
}

////////////////////////////////////////

#[derive(Clone,Copy)]
pub struct Sprite {
    pub data: usize,
    pub locView: Mvec,
    pub en: bool,
}

impl Default for Sprite {
  fn default () -> Sprite {
    Sprite{
      data: 0,
      locView: Mvec::new(WINDOW.width, WINDOW.height, 0, 0), // this is changed to spriteView modulus
      en:false
    }
  }
}

////////////////////////////////////////////////////////////////////////////////

const MEMORYSIZE: usize = 256*256;
pub const SPRITECOUNT: usize = 18;

pub const   TILE: Dim = Dim(8, 8);
pub const SPRITE: Dim = Dim(16, 16);

// World Field / Window View(port) / Frame(buffer) Raster / CRT
pub const WORLD:  Dim = Dim(64, 64);
pub const WINDOW: Dim = Dim(WORLD.width*TILE.width, WORLD.height*TILE.height);

pub struct Graphics {
    term: Term,
    pub rnd: Rnd,
    memory: [u8; MEMORYSIZE],
    memPtr: usize,
    c2b:    HashMap<char, u8>,
    world:   [u8; WORLD.volume],
    worldDirty:  [bool; WORLD.volume],
    pub sprites: [Sprite; SPRITECOUNT],

    pub spriteTileCenterAdj: Mvec,

    pub field: Mvec,
    pub spriteField: Mvec,

    pub view: Mvec,
    pub spriteView: Mvec,

    pub rasterView: Mvec,

    framebuff: [u8; WINDOW.volume],
    last:   [u8; WINDOW.volume],
    pub msg: String,
    buff: String
}

impl Graphics {
  pub fn new(term: Term) -> Graphics {
    Graphics{
      term,
      rnd:  Rnd::new(),
      memory:  [0; MEMORYSIZE],
      memPtr:  0,
      c2b:     HashMap::<char, u8>::new(),
      world:     [1; WORLD.volume],
      worldDirty:    [true; WORLD.volume],
      sprites:   [Sprite::default(); SPRITECOUNT],

      spriteTileCenterAdj: Mvec::new(0, 0,
        ((TILE.width as isize - SPRITE.width as isize) / 2) as usize ,
        ((TILE.height as isize - SPRITE.height as isize) / 2) as usize ),

      field: Mvec::new(WORLD.width, WORLD.height, 0, 0),
      spriteField: Mvec::new(WORLD.width, WORLD.height, 0, 0),

      view: Mvec::new(WINDOW.width, WINDOW.height, 0, 0),
      spriteView: Mvec::new(WINDOW.width, WINDOW.height, 0, 0),

      rasterView: Mvec{w:0, h:0, x:0, y:0},

      framebuff:   [0; WINDOW.volume],
      last:     [255;WINDOW.volume],
      buff: String::new(),

      msg:  String::new(),
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
        self.world[x+y*WORLD.width]=char2clr[&c] );
      y += 1;
    });
  }

  pub fn enableSprite (&mut self, sprite: usize) {
    self.sprites[sprite].en = true;
  }
  pub fn setLocViewFromSprite (&mut self, sprite: usize) {
    let s = &mut self.sprites[sprite];
    s.locView.w = self.spriteView.w;
    s.locView.h = self.spriteView.h;
  }

  fn getFieldTile (&self, x: usize, y: usize) -> u8 {
    self.world[x + y*WORLD.width]
  }

  pub fn getFieldCardinalTiles<F,R> (&self, xf: usize, yf: usize, f: F) -> [(R, usize, isize, isize); 4]
  where F: Fn(u8)->R
  {
    let row = yf*WORLD.width;
    [
      (f(self.world[(xf+1)%self.spriteField.w + row]),                        0, xf as isize + 1, yf as isize),
      (f(self.world[(xf+row+WORLD.width) % WORLD.volume]),             1, xf as isize,     yf as isize+1),
      (f(self.world[(xf+self.spriteField.w-1)%self.spriteField.w + row]),            2, xf as isize - 1, yf as isize),
      (f(self.world[(xf+row+WORLD.volume-WORLD.width) % WORLD.volume]),3, xf as isize,     yf as isize-1),
    ]
  }

  pub fn setFieldTile(&mut self, x: usize, y: usize, tile: u8) -> u8{
      let idx = x + y*WORLD.width;
      let old = self.world[idx];
      self.world[idx] = tile;
      old
  }

  pub fn setSpriteLocWindow(&mut self, sprite: usize, x: usize, y: usize) {
    self.sprites[sprite].locView.set(x, y);
  }
  pub fn shiftSprite(&mut self, sprite: usize, dir: usize, mag: usize) {
    self.sprites[sprite].locView.shift(dir, mag);
  }
  pub fn setSpriteIdx(&mut self, sprite: usize, p: usize) {
    self.sprites[sprite].data = p;
  }

  pub fn setFieldSize(&mut self, w: usize, h: usize) {
    self.field = Mvec::new(w, h, 0, 0);
    self.view = Mvec::new(w*TILE.width, h*TILE.height, 0, 0);
    self.spriteField = Mvec::new(
      w + (SPRITE.width + TILE.width - 1) / TILE.width,
      h + (SPRITE.height + TILE.height - 1) / TILE.height,
      0, 0);
    self.spriteView = Mvec::new(
      self.spriteField.w * TILE.width,
      self.spriteField.h * TILE.height,
      0, 0);
  }

  pub fn centerRasterView(&mut self, sprite: usize) { // modulo spriteView
    let x = self.sprites[sprite].locView.x();
    let y = self.sprites[sprite].locView.y();
    let w = min(self.term.w, self.view.w);
    let h = min(self.term.h, self.view.h);
    self.rasterView.w = w;
    self.rasterView.h = h;
    self.rasterView.x =
      min(
        max(0, ((x + SPRITE.width) % self.spriteView.w - SPRITE.width/2) as isize - w as isize / 2),
        (self.view.w-w) as isize)
      as usize;

    self.rasterView.y =
      min(
        max(0, ((y + SPRITE.height) % self.spriteView.h - SPRITE.height/2) as isize - h as isize / 2),
        (self.view.h-h) as isize)
      as usize;
    self.msg = format!("{}    ", x + 8 - w/2);
   }

  pub fn rasterizeTilesSprites(&mut self, dataTiles: usize) {
    // render tiles
    for fy in 0..self.field.h {
    for fx in 0..self.field.w {
      if !self.worldDirty[fx+fy*WORLD.width] { continue }
      self.worldDirty[fx+fy*WORLD.width] = false;
      let roffset = fx*TILE.width + fy*TILE.volume*WORLD.width;
      let mut ptr = self.getFieldTile(fx, fy) as usize * TILE.volume + dataTiles;
      for ty in 0..TILE.height {
      for tx in 0..TILE.width {
        self.framebuff [roffset + tx + ty*WINDOW.width] = self.memory [ptr];
        ptr += 1;
      }}
    }}

    // render sprites
    for s in 0..SPRITECOUNT {
      if ! self.sprites[s].en { continue }

      let xwindow = self.sprites[s].locView.x();
      let ywindow = self.sprites[s].locView.y();

      // dirty the tile bit
      let (wsf, hsf) = {
        (self.spriteField.w, self.spriteField.h)
      };
      let xsf = xwindow/TILE.width;
      let ysf = ywindow/TILE.height;
      self.worldDirty[ xsf%wsf    + (ysf%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+1)%wsf + (ysf%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+2)%wsf + (ysf%hsf)*WORLD.width] = true;

      self.worldDirty[ xsf%wsf    + ((1+ysf)%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+1)%wsf + ((1+ysf)%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+2)%wsf + ((1+ysf)%hsf)*WORLD.width] = true;

      self.worldDirty[ xsf%wsf    + ((2+ysf)%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+1)%wsf + ((2+ysf)%hsf)*WORLD.width] = true;
      self.worldDirty[(xsf+2)%wsf + ((2+ysf)%hsf)*WORLD.width] = true;

      let spritedata = self.sprites[s].data;
      for shy in 0..SPRITE.height {
          let y = (ywindow + shy) % self.spriteView.h;
          //if self.view.h <= y { continue }
          for swx in 0..SPRITE.width {
              let x = (xwindow + swx) % self.spriteView.w;
              //if self.view.w <= x { continue }
              match self.memory[spritedata + swx + shy * SPRITE.width] {
                0 => continue, // transparent pixel
                b => self.framebuff[x + y*WINDOW.width] = b
              }
          } // for x
      } // for y
    } // for s
  }

  pub fn printField(&mut self) {
    let w = self.rasterView.w;
    let h = self.rasterView.h;

    self.buff.clear();
    write!(self.buff, "\x1b[H\x1b[30m").ok();

    let mut idx = 0;
    let mut ridx =  self.rasterView.x + self.rasterView.y*WINDOW.width;
    let mut lastColor=0;
    let mut loc=(0,0);

    for y in 0..h {
      for x in 0..w {
        let b = self.framebuff[ridx];
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

    // debug dump dirty field
    //print!("\x1b[H");
    //for y in 0..WORLD.width {
    //  for x in 0..WORLD.height {
    //    print!("{}", if self.worldDirty[x+y*WORLD.width] { 1 } else { 0 })
    //  }
    //  println!("\r");
    //}

    if !self.msg.is_empty() { // Maybe overlay debug msg
      //self.msg = format!("{:?}", locCenter);
      print!("\x1b[H\x1b[37m{}", self.msg);
      //self.msg.clear();
    }
    print!("\x1b[{};{}H\x1b[37m🚽", h/2, w/2);
  }
} // impl Graphics

////////////////////////////////////////
