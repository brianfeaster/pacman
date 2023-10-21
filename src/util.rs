use std::{
  env,
  io::{Write, stdout}, thread, time::Duration,
};

////////////////////////////////////////

pub fn sleep (ms: u64)  {
  stdout().flush().ok();
  thread::sleep(Duration::from_millis(ms));
}

////////////////////////////////////////

//use std::io::stdin;
//pub fn readline() { stdin().read_line(&mut String::new()).ok(); }

////////////////////////////////////////

pub const fn neg_mod (n: usize, b: usize) -> usize { b - n }

pub struct Rnd (u16);

impl Rnd {
  pub fn new () -> Rnd { Rnd(0xfaec) }
  pub fn rnd (&mut self) -> u16 {
    self.0 ^= self.0>>7;
    self.0 ^= self.0<<9;
    self.0 ^= self.0>>13;
    self.0
  }
}

////////////////////////////////////////
pub struct Average {
  max: usize,
  count: usize,
  nums: Vec<usize>,
  idx: usize,
  sum: usize
}

impl Average {
  pub fn new (max: usize) -> Average { Average{max, count:0, nums:vec![0; max], idx:0, sum:0} }
  pub fn add (&mut self, num: usize) -> usize {
    self.sum += num - self.nums[self.idx];
    self.nums[self.idx] = num;
    self.idx = (self.idx + 1) % self.max;
    if self.count < self.max { self.count += 1 }
    self.sum / self.count
  }
}
////////////////////////////////////////

pub struct Term {
  pub h: usize,
  pub w: usize
}

impl Term {
  pub fn new() -> Term {
    let mut args = env::args().skip(1).take(2).flat_map(|s| s.parse::<usize>());
    Term {
      h: args.next().unwrap_or(25),
      w: args.next().unwrap_or(80)
    }
  }
}

////////////////////////////////////////

#[macro_export]
macro_rules! IF {
    ($p:expr, $t:expr, $f:expr) => (if $p { $t } else { $f })
}
