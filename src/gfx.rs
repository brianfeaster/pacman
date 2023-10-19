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

// Includes non-visible right/bottom borders
const RASTERWIDTH:  usize = TILEWIDTH * FIELDWIDTH;
const RASTERHEIGHT: usize = TILEHEIGHT * FIELDHEIGHT;

pub const SPRITEFIELDWIDTH: usize  = FIELDWIDTH + SPRITEWIDTH/TILEWIDTH;
pub const SPRITEFIELDHEIGHT: usize = FIELDHEIGHT + SPRITEHEIGHT/TILEHEIGHT;

pub const SPRITERASTERWIDTH: usize  = SPRITEFIELDWIDTH * TILEWIDTH;
pub const SPRITERASTERHEIGHT: usize = SPRITEFIELDHEIGHT * TILEHEIGHT;

////////////////////////////////////////

#[derive(Clone,Copy,Default,PartialEq,Debug)]
pub struct Loc {
  pub x: usize,
  pub y: usize,
}

#[derive(Clone,Copy,Default)]
pub struct Sprite {
    pub data: usize,
    pub loc: Loc,
    pub en: bool,
}

pub struct Graphics {
    term: Term,
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
    pub topleft: Loc
}

impl Graphics {
  pub fn new(term: Term) -> Graphics {
    Graphics{
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
      topleft: Loc::default()
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
      (f(self.field[(xf+y+FIELDVOLUME-FIELDWIDTH) % FIELDVOLUME]),0, xf as isize,     yf as isize-1),
      (f(self.field[(xf+FIELDWIDTH-1)%FIELDWIDTH + y]),           1, xf as isize - 1, yf as isize),
      (f(self.field[(xf+1)%FIELDWIDTH + y]),                      2, xf as isize + 1, yf as isize),
      (f(self.field[(xf+y+FIELDWIDTH) % FIELDVOLUME]),            3, xf as isize,     yf as isize+1),
    ]
  }

  pub fn setFieldTile(&mut self, loc: Loc, tile: u8) -> u8{
      let old = self.field[loc.y*FIELDWIDTH+loc.x];
      self.field[loc.y*FIELDWIDTH+loc.x] = tile;
      old
  }

  pub fn setSpriteLoc(&mut self, i: usize, loc: Loc) {
    self.sprites[i].loc = loc;
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

      let slocy = self.sprites[s].loc.y;
      let slocx = self.sprites[s].loc.x;

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
  pub fn printField(&mut self, locCenter: Loc) {
    let w = min(self.term.w, RASTERWIDTH);
    let h = min(self.term.h, RASTERHEIGHT);
    let mut buff = String::new();
    let mut lastColor=0;
    write!(buff, "\x1b[H\x1b[30m").ok();
    let mut loc=(0,0);
    self.topleft = Loc{
      x: min(max(0, ((locCenter.x+16)%SPRITERASTERWIDTH -8-(w>>1))as isize), (RASTERWIDTH-w)  as isize) as usize,
      y: min(max(0, ((locCenter.y+16)%SPRITERASTERHEIGHT-8-(h>>1))as isize), (RASTERHEIGHT-h) as isize) as usize,
    };
    let mut idx = 0;
    let mut ridx = self.topleft.y*RASTERWIDTH + self.topleft.x;
    for y in 0..h {
      for x in 0..w {
        let b = self.raster[ridx];
        let l = self.last[idx];
        if b!=l {
          // Update cursor
          if loc != (x, y) { write!(buff, "\x1b[{};{}H", y+1,x+1).ok(); }
          if lastColor != b {
            if 0==b {
              write!(buff, " ").ok();
            } else {
              write!(buff, "\x1b[38;5;{}m{BLK}", b).ok();
              lastColor = b;
            }
          } else {
            if 0==b {
              write!(buff, " ").ok();
            } else {
              write!(buff, "{BLK}").ok();
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
     self.msg = format!("{:?}", locCenter);
    <Stdout as io::Write>::write_all(&mut stdout(), buff.as_bytes()).ok();
    print!("\x1b[H\x1b[37m\x1b[K{}\x1b[H", self.msg);
    //self.msg.clear();
    //print!("\x1b[{};{}H\x1b[37m@", h/2, w/2);
  }
} // impl Graphics

////////////////////////////////////////
