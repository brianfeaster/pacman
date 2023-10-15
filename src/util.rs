use std::{
  env,
  io::{Write, stdout, stdin}, thread, time::Duration,
};

////////////////////////////////////////

pub fn sleep (ms: u64)  {
  stdout().flush().ok();
  thread::sleep(Duration::from_millis(ms));
}

////////////////////////////////////////

pub fn readline() { stdin().read_line(&mut String::new()).ok(); }

////////////////////////////////////////

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
