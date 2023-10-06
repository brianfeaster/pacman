#![allow(non_snake_case)]


use std::{
  cmp::{max, min},
  collections::HashMap,
  fmt::Write,
  io::{self, stdout, Stdout}
};

mod data;
mod util;
use data::{setData};
pub use util::{sleep, readline, Rnd, Term};

////////////////////////////////////////

// Pixels ᗣ ᗧ · ⋅ • ● ⬤
pub const BLKA :&str = "\x1b[3m "; // Inverted ASCII space
pub const BLKB :char = '█'; // full block 2588
pub const BLK   :char = '▉'; // left 7/8th block 2589
pub const BLKD :char = '◼'; // black medium square 25fc
pub const BLKE :char = '▮'; // black verticle rectangle 25ae
pub const BLKF :char = '▪'; // black small square 25aa

////////////////////////////////////////

const TILEWIDTH: usize = 8;
const TILEHEIGHT: usize = 8;
const TILEVOLUME: usize = TILEWIDTH*TILEHEIGHT;
const TILECOUNT: usize = 128;

// pacman field 28x36
const FIELDWIDTH: usize=28;
const FIELDHEIGHT: usize=36;
const FIELDVOLUME: usize=FIELDWIDTH*FIELDHEIGHT;

const SPRITEWIDTH: usize=16;
const SPRITEHEIGHT: usize=16;
const SPRITEVOLUME: usize=SPRITEWIDTH*SPRITEHEIGHT;
const SPRITECOUNT: usize = 8;

type TileData   = [u8; TILEVOLUME];
type SpriteData = [u8; SPRITEVOLUME];

#[derive(Clone,Copy,Default)]
struct Sprite {
    id: usize,
    x: usize,
    y: usize,
    en: bool,
}

pub struct Video {
    tiledata:   [TileData;   TILECOUNT],
    spritedata: [SpriteData; TILECOUNT],
    field:   [u8;  FIELDVOLUME],
    sprites: [Sprite; SPRITECOUNT],
    raster: [u8; TILEVOLUME*FIELDVOLUME],
    last: [u8; TILEVOLUME*FIELDVOLUME],
    rnd: Rnd
}

impl Video {
  fn new() -> Video {
    Video{
      tiledata:  [[0; TILEVOLUME];   TILECOUNT],
      spritedata:[[0; SPRITEVOLUME]; TILECOUNT],
      sprites:   [Sprite::default(); SPRITECOUNT],
      field:     [63; FIELDVOLUME],
      raster:    [0;  TILEVOLUME*FIELDVOLUME],
      last:      [255;TILEVOLUME*FIELDVOLUME],
      rnd:       Rnd::new()
    }
  }
  fn getField (&self, x: usize, y: usize) -> u8 { self.field[y*FIELDWIDTH+x] }
  fn spriteData (&self, id: usize) -> &SpriteData {
    &self.spritedata[self.sprites[id].id]
  }

  pub fn setFieldTile(&mut self, id: u8, x: usize, y: usize) {
      self.field[y*FIELDWIDTH+x]=id
  }
  fn setTileData(&mut self, id: usize, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.tiledata[id][i]=hm[&c];
            i += 1;
        })
    });
  }
  fn setSpriteData(&mut self, id: usize, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.spritedata[id][i]=hm[&c];
            i += 1;
        })
    });
  }
  fn setFieldData(&mut self, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.field[i]=hm[&c];
            i += 1;
        })
    });
  }
  fn setSpriteLoc(&mut self, i: usize, x: usize, y: usize) {
    self.sprites[i].x = x;
    self.sprites[i].y = y;
  }
  fn setSpriteIdx(&mut self, i: usize, o: usize) {
    self.sprites[i].id = o;
  }
  fn rasterizeField (&mut self) {
    for fy in 0..FIELDHEIGHT { for fx in 0..FIELDWIDTH {
      for cy in 0..TILEHEIGHT {
        for cx in 0..TILEWIDTH {
          self.raster[
            fy*TILEVOLUME*FIELDWIDTH    + fx*TILEWIDTH
            + cy*TILEWIDTH*FIELDWIDTH + cx
          ] = self.tiledata[self.field[fy*FIELDWIDTH+fx] as usize][cy*TILEWIDTH+cx]
        }
      }
    } }

    for s in 0..SPRITECOUNT {
        if self.sprites[s].en { for cy in 0..SPRITEHEIGHT {
          for cx in 0..SPRITEWIDTH {
            let b = self.spritedata[self.sprites[s].id][cy*SPRITEWIDTH+cx];
            if 0 < b {
              self.raster[
                self.sprites[s].y*TILEWIDTH*FIELDWIDTH + self.sprites[s].x
                + cy*TILEWIDTH*FIELDWIDTH + cx
              ] = b;
            }
          }
        } }
    }
  }
  fn dumpField(&mut self, mut w: usize, mut h: usize, fx: usize, fy: usize) {
    w = min(w, TILEWIDTH*FIELDWIDTH);
    h = min(h, TILEHEIGHT*FIELDHEIGHT);
    let mut buff = String::new();
    let mut lastColor=0;
    write!(buff, "\x1b[H\x1b[30m").ok();
    let mut loc=(0,0);
    for y in 0..h {
      if 0 != y { write!(buff, "\n").ok(); }
      for x in 0..w {

        let top =
          min(
            max(0, (fy + 4) as isize - (h as isize >>1) )as usize,
            FIELDHEIGHT*TILEHEIGHT - h);

        let left =
          min(
            max(0, (fx + 4) as isize - (w as isize >>1) )as usize,
            FIELDWIDTH*TILEWIDTH - w);

        let b = self.raster[(y+top)*TILEWIDTH*FIELDWIDTH+(x+left)];
        let l = self.last[y*TILEWIDTH*FIELDWIDTH+x];

        if b!=l {
          if loc != (x, y) {
              write!(buff, "\x1b[{};{}H", y+1,x+1).ok();
          }
          if lastColor != b {
             write!(buff, "\x1b[38;5;{}m{BLK}", b).ok();
             lastColor = b;
          } else {
             write!(buff, "{BLK}").ok();
          }
          self.last[y*TILEWIDTH*FIELDWIDTH+x] = b;
          loc = (x+1, y);
        }
      }
    }
    <Stdout as io::Write>::write_all(&mut stdout(), buff.as_bytes()).ok();
  }

  pub fn debugSprite(&self, id: usize) {
    let cell = self.spriteData(id);
    let mut i = 0;
    for y in 0..SPRITEHEIGHT {
      if 0 != y { println!() }
      for _ in 0..SPRITEWIDTH {
        let b = cell[i];
        if 0 == b {
           print!("\x1b[0;40;37m.")
        } else {
           print!("\x1b[0;40;38;5;{}m{BLK}", b)
        }
        i += 1;
      }
    }
  }

} // impl Video

fn randir (vid: &mut Video, x: usize, y: usize, mut lastdir: usize) -> usize {
   let validDirs = [
       vid.getField(x, y-1)<3,
       vid.getField(x-1, y)<3,
       vid.getField(x+1, y)<3,
       vid.getField(x, y+1)<3,
   ];

   lastdir = [3, 2, 1, 0][lastdir];
   loop { match vid.rnd.rnd()%4 {
     0 => if lastdir!=0 && validDirs[0] { return 0 },
     1 => if lastdir!=1 && validDirs[1] { return 1 },
     2 => if lastdir!=2 && validDirs[2] { return 2 },
     3 => if lastdir!=3 && validDirs[3] { return 3 },
     _ => ()
   } }
}

const DMX :[isize;4] = [0, -1, 1, 0];
const DMY :[isize;4] = [-1, 0, 0, 1];

struct Entity {
    sprite: usize,
    data: usize,
    fx: usize,
    fy: usize,
    dir: usize,
    tick: usize
}

impl Entity {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Entity {
    Entity{sprite, data, fx, fy, dir, tick: 0}
  }
  fn tick (&mut self, vid: &mut Video) {
    vid.sprites[self.sprite].en=true;
    let inc = self.tick%8;
    let x = 8*self.fx as isize+ DMX[self.dir]*inc as isize - 4;
    let y = 8*self.fy as isize + DMY[self.dir]*inc as isize - 4;
    vid.setSpriteIdx(self.sprite, self.data+self.dir*2 + self.tick/8%2);
    vid.setSpriteLoc(self.sprite, x as usize, y as usize);
    if 7 == inc {
       self.fx = (self.fx as isize + DMX[self.dir]) as usize;
       self.fy = (self.fy as isize + DMY[self.dir]) as usize;
       self.dir = randir(vid, self.fx as usize, self.fy as usize, self.dir);
    }
    self.tick += 1;
  }
}

struct Pukman {
    sprite: usize,
    data: usize,
    fx: usize,
    fy: usize,
    dir: usize,
    tick: usize,
    fx2: usize,
    fy2: usize,
}

impl Pukman {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Self {
    Self{sprite, data, fx, fy, dir, tick:0, fx2:0, fy2:0}
  }
  fn tick (&mut self, vid: &mut Video) {
    vid.sprites[self.sprite].en=true;
    let inc = self.tick%8;
    let x = 8*self.fx as isize + DMX[self.dir]*inc as isize - 4;
    let y = 8*self.fy as isize + DMY[self.dir]*inc as isize - 4;
    self.fx2 = x as usize;
    self.fy2 = y as usize;

    vid.setSpriteIdx(self.sprite,
      (self.data as isize + match self.tick%4 {
        0 => 0,
        1 => self.dir as isize * 2 + 1,
        2 => self.dir as isize * 2 + 2,
        3 => self.dir as isize * 2 + 1,
        _ => 0
      }) as usize
    );

    vid.setSpriteLoc(self.sprite, x as usize, y as usize);
    if 7 == inc {
       self.fx = (self.fx as isize + DMX[self.dir]) as usize;
       self.fy = (self.fy as isize + DMY[self.dir]) as usize;
       self.dir = randir(vid, self.fx as usize, self.fy as usize, self.dir);
    }
    if 0 == self.tick%4 {vid.setFieldTile(0, self.fx, self.fy); }
    self.tick += 1;
  }
}

fn main() {
    let term = Term::new();
    let mut vid = Video::new();
    setData(&mut vid);

    //vid.setFieldTile(2, 0, 0);

    let mut blinky = Entity::new(0, 0,  1, 4, 2);
    let mut pinky  = Entity::new(1, 8,  12, 4, 3);
    let mut inky   = Entity::new(2, 16, 1, 8, 0);
    let mut clyde  = Entity::new(3, 24, 12, 8, 1);
    let mut pukman = Pukman::new(4, 32, 3, 4, 2); // bottom 1 29 3
    //let mut pukmana = Pukman::new(5, 32, 6, 4, 3);
    //let mut pukmanb = Pukman::new(6, 32, 6, 4, 3);
    //let mut pukmanc = Pukman::new(7, 32, 6, 4, 3);

    print!("\x1bc\x1b[0;30;40m\x1b[H\x1b[J");

    loop {
        blinky.tick(&mut vid);
        pinky.tick(&mut vid);
        inky.tick(&mut vid);
        clyde.tick(&mut vid);
        pukman.tick(&mut vid);
        //pukmana.tick(&mut vid);
        //pukmanb.tick(&mut vid);
        //pukmanc.tick(&mut vid);
        vid.rasterizeField();
        vid.dumpField(term.w, term.h, pukman.fx2, pukman.fy2);
        sleep(40);
    }
    //vid.debugSprite(0);
}
