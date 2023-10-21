use std::{
  fmt::Write,
  cmp::{max, min},
  collections::HashMap,
  io::{self, stdout, Stdout},
};

use crate::util::{Rnd, Term, neg_mod};

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

const MEMORYSIZE: usize = 256*256; // 64k

pub const TILEWIDTH: usize  = 8;
pub const TILEHEIGHT: usize = 8;
const TILEVOLUME: usize = TILEWIDTH*TILEHEIGHT;

// tile field 28x36
pub const FIELDWIDTH: usize  = 28;
pub const FIELDHEIGHT: usize = 36;
const FIELDVOLUME: usize = FIELDWIDTH*FIELDHEIGHT;

pub const SPRITEWIDTH: usize = 16;
pub const SPRITEHEIGHT: usize= 16;
pub const SPRITEVOLUME: usize= SPRITEWIDTH*SPRITEHEIGHT;
const SPRITECOUNT: usize = 16;

// Amount to center sprite over its tile location
pub const SPRITECENTERX: usize = neg_mod((SPRITEWIDTH-TILEWIDTH)/2, SPRITERASTERWIDTH);
pub const SPRITECENTERY: usize = neg_mod((SPRITEHEIGHT-TILEHEIGHT)/2, SPRITERASTERHEIGHT);

pub const SPRITEFIELDWIDTH: usize  = FIELDWIDTH + SPRITEWIDTH/TILEWIDTH;
pub const SPRITEFIELDHEIGHT: usize = FIELDHEIGHT + SPRITEHEIGHT/TILEHEIGHT;

const RASTERWIDTH:  usize = TILEWIDTH * FIELDWIDTH;
const RASTERHEIGHT: usize = TILEHEIGHT * FIELDHEIGHT;

// Includes non-visible right/bottom borders
pub const SPRITERASTERWIDTH: usize  = RASTERWIDTH + SPRITEWIDTH;
pub const SPRITERASTERHEIGHT: usize = RASTERHEIGHT + SPRITEHEIGHT;

////////////////////////////////////////

#[derive(Debug, Clone,Copy)]
pub struct Mloc {
  w: usize,
  h: usize,
  pub x: usize,
  pub y: usize,
}

impl Mloc {
  pub const UP: usize = 0;
  pub fn equal (&self, o: &Mloc) -> bool {
    self.x() == o.x() && self.y() == o.y()
  }
  pub fn new (w: usize, h: usize, x: usize, y: usize) -> Mloc { Mloc{w, h, x:(w<<32)+x, y:(h<<32)+y} }
  pub fn area (&self) -> usize { self.w * self.h }
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
  pub fn next (&self, dir: usize) -> Mloc {
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

#[derive(Clone,Copy,Default,PartialEq, Debug)]
pub struct Loc {
  pub x: usize,
  pub y: usize,
}

impl Loc {
  fn new (x: usize, y: usize) -> Loc { Loc{x, y} }
}

#[derive(Clone,Copy)]
pub struct Sprite {
    pub data: usize,
    pub mloc: Mloc,
    pub en: bool,
}

impl Default for Sprite {
  fn default () -> Sprite {
    Sprite{data:0, mloc:Mloc{w:SPRITERASTERWIDTH, h:SPRITERASTERHEIGHT, x:0, y:0}, en:false}
  }
}

pub struct Graphics {
    term: Term,
    pub viewportSize: Loc,
    memory: [u8; MEMORYSIZE],
    memPtr: usize,
    c2b:    HashMap<char, u8>,
    field:   [u8; FIELDVOLUME],
    update:  [bool; FIELDVOLUME],
    pub sprites: [Sprite; SPRITECOUNT],
    raster: [u8; TILEVOLUME*FIELDVOLUME],
    last:   [u8; TILEVOLUME*FIELDVOLUME],
    pub rnd: Rnd,
    pub msg: String,
    pub topleft: Loc,
    buff: String
}

impl Graphics {
  pub fn new(term: Term) -> Graphics {
    Graphics{
      viewportSize: Loc{x:min(term.w, RASTERWIDTH), y:min(term.h, RASTERHEIGHT)},
      term,
      memory:  [0; MEMORYSIZE],
      memPtr:  0,
      c2b:     HashMap::<char, u8>::new(),
      field:     [63; FIELDVOLUME],
      update:    [true; FIELDVOLUME],
      sprites:   [Sprite::default(); SPRITECOUNT],
      raster:   [0;  (TILEVOLUME*FIELDVOLUME)],
      last:     [255;(TILEVOLUME*FIELDVOLUME)],
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
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.field[i]=hm[&c];
            i += 1;
        })
    });
  }
  pub fn _getTile (&self, loc: Loc) -> u8 {
    self.field[loc.x + loc.y*FIELDWIDTH]
  }

  fn getFieldTile (&self, x: usize, y: usize) -> u8 {
    self.field[x + y*FIELDWIDTH]
  }

  pub fn getFieldTiles<F,R> (&self, xf: usize, yf: usize, f: F) -> [(R, usize, isize, isize); 4]
  where F: Fn(u8)->R
  {
    let y = yf*FIELDWIDTH;
    [
      (f(self.field[(xf+1)%FIELDWIDTH + y]),                      0, xf as isize + 1, yf as isize),
      (f(self.field[(xf+y+FIELDWIDTH) % FIELDVOLUME]),            1, xf as isize,     yf as isize+1),
      (f(self.field[(xf+FIELDWIDTH-1)%FIELDWIDTH + y]),           2, xf as isize - 1, yf as isize),
      (f(self.field[(xf+y+FIELDVOLUME-FIELDWIDTH) % FIELDVOLUME]),3, xf as isize,     yf as isize-1),
    ]
  }

  pub fn setFieldTile(&mut self, x: usize, y: usize, tile: u8) -> u8{
      let old = self.field[x+y*FIELDWIDTH];
      self.field[x+y*FIELDWIDTH] = tile;
      old
  }

  pub fn setSpriteLoc(&mut self, i: usize, x: usize, y:usize) {
    self.sprites[i].mloc.x = x;
    self.sprites[i].mloc.y = y;
  }
  pub fn setSpriteIdx(&mut self, i: usize, p: usize) {
    self.sprites[i].data = p;
  }
  pub fn rasterizeTilesSprites(&mut self, dataTiles: usize) {
    // tiles
    for fy in 0..FIELDHEIGHT {
    for fx in 0..FIELDWIDTH {
      if !self.update[fx+fy*FIELDWIDTH] { continue }
      self.update[fx+fy*FIELDWIDTH]=false;
      let roffset = fy*TILEVOLUME*FIELDWIDTH + fx*TILEWIDTH;
      let mut ptr = self.getFieldTile(fx, fy) as usize * TILEVOLUME + dataTiles;
      for cy in 0..TILEHEIGHT {
      for cx in 0..TILEWIDTH {
        self.raster [roffset + cx + cy*RASTERWIDTH] = self.memory [ptr];
        ptr += 1;
      }}
    }}

    // sprites
    for s in 0..SPRITECOUNT {
      if ! self.sprites[s].en { continue }

      let slocy = self.sprites[s].mloc.y();
      let slocx = self.sprites[s].mloc.x();

      // dirty tile bit
      let mut xsf = slocx/TILEWIDTH;
      let mut ysf = slocy/TILEHEIGHT;
      if FIELDWIDTH  <= xsf { xsf=FIELDWIDTH-1 }
      if FIELDHEIGHT <= ysf { ysf=FIELDHEIGHT-1 }
      self.update[ xsf               + ysf%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + ysf%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + ysf%FIELDHEIGHT*FIELDWIDTH] = true;

      self.update[ xsf               + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;

      self.update[ xsf               + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;

      let spritedata = self.sprites[s].data;

      for cy in 0..SPRITEHEIGHT {
          let y = (slocy + cy) % SPRITERASTERHEIGHT;
          if RASTERHEIGHT <= y { continue }
          for cx in 0..SPRITEWIDTH {
              let x = (slocx + cx) % SPRITERASTERWIDTH;
              if RASTERWIDTH <= x { continue }
              match self.memory[spritedata + cx+cy*SPRITEWIDTH] {
                0 => continue,
                b => self.raster[x+y*RASTERWIDTH] = b
              } // match
          } // for x
      } // for y
    } // for s

  }

  pub fn alignViewport(&mut self, locCenter: &Mloc) {
    let w = self.viewportSize.x;
    let h = self.viewportSize.y;

    self.topleft = Loc{
      x: min(max(0, (locCenter.x()+SPRITEWIDTH/2-(w>>1))as isize), (RASTERWIDTH-w)  as isize) as usize,
      y: min(max(0, (locCenter.y()+SPRITEHEIGHT/2-(h>>1))as isize), (RASTERHEIGHT-h) as isize) as usize,
    }
  }

  pub fn printField(&mut self) {
    let w = self.viewportSize.x;
    let h = self.viewportSize.y;

    self.buff.clear();
    write!(self.buff, "\x1b[H\x1b[30m").ok();

    let mut idx = 0;
    let mut ridx =  self.topleft.x + self.topleft.y*RASTERWIDTH;
    let mut lastColor=0;
    let mut loc=(0,0);

    for y in 0..h {
      for x in 0..w {
        let b = self.raster[ridx];
        let l = self.last[idx];
        if b!=l {
          // Update cursor
          if loc != (x, y) { write!(self.buff, "\x1b[{};{}H", y+1,x+1).ok(); }
          if lastColor != b {
            if 0==b {
              write!(self.buff, " ").ok();
            } else {
              write!(self.buff, "\x1b[38;5;{}m{BLK}", b).ok();
              lastColor = b;
            }
          } else {
            if 0==b {
              write!(self.buff, " ").ok();
            } else {
              write!(self.buff, "{BLK}").ok();
            }
          }
          self.last[idx] = b;
          loc = (x+1, y);
        }
        ridx += 1;
        idx += 1;
      }
      ridx += RASTERWIDTH-w;
    }
    <Stdout as io::Write>::write_all(&mut stdout(), self.buff.as_bytes()).ok();

    //if self.msg.is_empty() { // Maybe overlay debug msg
    //  //self.msg = format!("{:?}", locCenter);
    //  print!("\x1b[H\x1b[37m{}", self.msg);
    //  print!("\x1b[{};{}H\x1b[37m🚽", h/2, w/2);
    //  //self.msg.clear();
    //}
  }
} // impl Graphics

////////////////////////////////////////
