//  --                                                          ; {{{1
//
//  File        : src/main.rs
//  Maintainer  : Felix C. Stegerman <flx@obfusk.net>
//  Date        : 2020-06-26
//
//  Copyright   : Copyright (C) 2020  Felix C. Stegerman
//  Version     : v0.1.2
//  License     : GPLv3+
//
//  --                                                          ; }}}1

const VERSION: &str = "0.1.2";

const HELP: &str = "\
Usage: proudcat [OPTIONS] [FILES]...

  proudcat-rust - cat + rainbow

  Flags: {}.

  Aliases: lgbt, aro, ace, bi, enby, nb, pan, trans.

Options:
  -f, --flag TEXT                 Choose which flags to use; default: pride.
  -b, --background                Set background colour (instead of
                                  foreground).
  -t, --truecolor / -T, --no-truecolor
                                  Explicitly enable/disable 24-bit colours;
                                  default: autodetect.
  --light                         Light terminal.
  --demo                          Demonstrate flags.
  --version                       Show the version and exit.
  --help                          Show this message and exit.";

const FLAGS: &str = "\
  pride agender aromantic asexual bisexual genderfluid
  genderqueer lesbian nonbinary pansexual polysexual transgender";

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter;
use std::process;

macro_rules! oops {
  ($msg:expr)   => {{ oops($msg); panic!() }};
  ($($arg:tt)*) => {{ oops(format!($($arg)*)); panic!() }}
}

fn oops(msg: String) {
  eprintln!("Error: {}", msg);
  process::exit(1)
}

#[derive(Clone, Copy)]
struct Rgb(u8, u8, u8);

fn rgbto8(c: &Rgb) -> u8 {
  (16 + (c.0 as u16)*6/256*36 + (c.1 as u16)*6/256*6 + (c.2 as u16)*6/256) as u8
}

// https://en.wikipedia.org/wiki/Pride_flag#Gallery
fn flag2colours(flag: &str) -> Vec<Rgb> {                     //  {{{1
  let f = match flag {
    "lgbt" => "pride", "aro" => "aromantic", "ace" => "asexual",
    "bi" => "bisexual", "enby" => "nonbinary", "nb" => "nonbinary",
    "pan" => "pansexual", "trans" => "transgender", _ => flag
  };
  let cs = match f {
    "pride"       => vec![Rgb(228, 3, 3), Rgb(255, 140, 0), Rgb(255, 237, 0),
                          Rgb(0, 128, 38), Rgb(0, 77, 255), Rgb(117, 7, 135)],
    "agender"     => vec![Rgb(0, 0, 0), Rgb(185, 185, 185), Rgb(255, 255, 255),
                          Rgb(184, 244, 131),
                          Rgb(255, 255, 255), Rgb(185, 185, 185), Rgb(0, 0, 0)],
    "aromantic"   => vec![Rgb(61, 165, 66), Rgb(167, 211, 121),
                          Rgb(255, 255, 255), Rgb(169, 169, 169), Rgb(0, 0, 0)],
    "asexual"     => vec![Rgb(0, 0, 0), Rgb(163, 163, 163),
                          Rgb(255, 255, 255), Rgb(128, 0, 128)],
    "bisexual"    => vec![Rgb(214, 2, 112), Rgb(214, 2, 112), Rgb(155, 79, 150),
                          Rgb(0, 56, 168), Rgb(0, 56, 168)],
    "genderfluid" => vec![Rgb(255, 117, 162), Rgb(255, 255, 255),
                          Rgb(190, 24, 214), Rgb(0, 0, 0), Rgb(51, 62, 189)],
    "genderqueer" => vec![Rgb(181, 126, 220), Rgb(255, 255, 255), Rgb(74, 129, 35)],
    "lesbian"     => vec![Rgb(213, 45, 0), Rgb(255, 154, 86), Rgb(255, 255, 255),
                          Rgb(211, 98, 164), Rgb(163, 2, 98)],
    "nonbinary"   => vec![Rgb(255, 244, 48), Rgb(255, 255, 255),
                          Rgb(156, 89, 209), Rgb(0, 0, 0)],
    "pansexual"   => vec![Rgb(255, 33, 140), Rgb(255, 216, 0), Rgb(33, 177, 255)],
    "polysexual"  => vec![Rgb(246, 28, 185), Rgb(7, 213, 105), Rgb(28, 146, 246)],
    "transgender" => vec![Rgb(91, 206, 250), Rgb(245, 169, 184), Rgb(255, 255, 255),
                          Rgb(245, 169, 184), Rgb(91, 206, 250)],
    _ => oops!("unknown flag: {}", flag)
  };
  if cs.len() > 3 { cs } else {
    cs.iter().flat_map(|&c| iter::repeat(c).take(2)).collect()
  }
}                                                             //  }}}1

const BLACK: Rgb = Rgb(0, 0, 0);
const WHITE: Rgb = Rgb(255, 255, 255);

fn colours(flags: &Vec<String>) -> Vec<Rgb> {
  flags.iter().flat_map(|f| flag2colours(f)).collect()
}

fn detect_truecolor() -> bool {
  let ct = env::var("COLORTERM").unwrap_or("".to_string());
  ct.contains("truecolor") || ct.contains("24bit")
}

fn with_colour(bg: bool, tc: bool, c: &Rgb, s: &str, clear: bool) -> String {
  let (sc, rc)  = if bg { (48, 49) } else { (38, 39) };
  let cl        = if bg && clear { "\x1b[2K" } else { "" };
  let code      = if tc { format!("2;{};{};{}", c.0, c.1, c.2) }
                   else { format!("5;{}", rgbto8(c)) };
  format!("\x1b[{};{}m{}{}\x1b[{};m", sc, code, cl, s, rc)
}

fn colour(bg: bool, tc: bool, li: bool, c: &Rgb, s: &str, clear: bool)
          -> String {
  let da = if li { bg } else { !bg };
  let t = if (da && brightness(c) < 0.20) || (!da && brightness(c) > 0.75) {
    with_colour(!bg, tc, if da { &WHITE } else { &BLACK }, s, clear)
  } else { s.to_string() };
  with_colour(bg, tc, c, t.as_ref(), clear)
}

fn brightness(c: &Rgb) -> f32 {
  (((c.0 as u32) * 299 + (c.1 as u32) * 587 + (c.2 as u32) * 114) / 1000)
    as f32 / 255f32
}

struct Options {
  flags: Vec<String>, bg: bool, tc: bool, light: bool, demo: bool
}

fn parse_args() -> (Options, Vec<String>) {                   //  {{{1
  let mut flags = Vec::new();
  let mut bg    = false;
  let mut tc    = detect_truecolor();
  let mut light = false;
  let mut demo  = false;
  let mut files = Vec::new();
  let mut done  = false;
  let mut flag  = None;
  for arg in env::args().skip(1).collect::<Vec<String>>() {
    if flag.is_some() {
      for f in arg.split(",") {
        flags.push(f.to_string())
      }
      flag = None
    } else if done {
      files.push(arg)
    } else {
      match arg.as_str() {
        "--"                    => done   = true,
        "-f" | "--flag"         => flag   = Some(arg),
        "-b" | "--background"   => bg     = true,
        "-t" | "--truecolor"    => tc     = true,
        "-T" | "--no-truecolor" => tc     = false,
        "--light"               => light  = true,
        "--demo"                => demo   = true,
        "--version" => {
          println!("proudcat-rust, version {}", VERSION);
          process::exit(0)
        }
        "--help" => {
          let fs = FLAGS.lines().map(|l|
            l.split_whitespace().collect::<Vec<_>>().join(", ")
          ).collect::<Vec<_>>().join(",\n  ");
          println!("{}", HELP.replace("{}", &fs));
          process::exit(0)
        }
        _ => if arg.starts_with("-") {
          oops!("no such option: {}", arg)
        } else {
          files.push(arg)
        }
      }
    }
  }
  if let Some(f) = flag {
    oops!("{} option requires an argument", f)
  }
  if flags.is_empty() { flags.push("pride".to_string()) }
  if files.is_empty() { files.push("-".to_string()) }
  (Options { flags, bg, tc, light, demo }, files)
}                                                             //  }}}1

// TODO
fn stdout_isatty() -> bool {
  if !cfg!(unix) { true } else {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
  }
}

fn main() {
  let (tty, stdin)  = (stdout_isatty(), io::stdin());
  let (opts, files) = parse_args();
  let clrs          = colours(&opts.flags);
  let mut it        = clrs.iter().cycle();
  if opts.demo {
    for flag in FLAGS.split_whitespace() {
      println!("┌{}┐", "─".repeat(flag.len()));
      for c in flag2colours(flag) {
        let s = colour(opts.bg, opts.tc, opts.light, &c, flag, false);
        println!("│{}│", s)
      }
      println!("└{}┘", "─".repeat(flag.len()))
    }
  } else {
    for file in files {
      let mut line = String::new();
      let mut bufr = if &file == "-" {
        Box::new(stdin.lock()) as Box<dyn BufRead>
      } else {
        Box::new(io::BufReader::new(
          File::open(file).unwrap_or_else(|e| oops!(e.to_string()))
        ))
      };
      while bufr.read_line(&mut line).unwrap() != 0 {
        let sline = line.trim();
        if sline.is_empty() && !(opts.bg && tty) {
          print!("{}", line)
        } else {
          let s = colour(opts.bg, opts.tc, opts.light,
                         it.next().unwrap(), sline, true);
          let i = sline.chars().next().and_then(|c| line.find(c))
                                      .unwrap_or(0);
          print!("{}{}{}", line[..i].to_string(), s,
                 line[i+sline.len()..].to_string())
        }
        line.clear()
      }
    }
  }
}

// vim: set tw=70 sw=2 sts=2 et fdm=marker :
