//  --                                                          ; {{{1
//
//  File        : src/main.rs
//  Maintainer  : Felix C. Stegerman <flx@obfusk.net>
//  Date        : 2020-06-23
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

  Currently avaliable flags: pride.

Options:
  -f, --flag TEXT                 Choose which flags to use; default: pride.
  -b, --background                Set background colour (instead of
                                  foreground).
  -t, --truecolor / -T, --no-truecolor
                                  Explicitly enable/disable 24-bit colours;
                                  default: autodetect.
  --version                       Show the version and exit.
  --help                          Show this message and exit.";

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;

macro_rules! oops {
  ($msg:expr)   => {{ oops($msg); panic!() }};
  ($($arg:tt)*) => {{ oops(format!($($arg)*)); panic!() }}
}

fn oops(msg: String) {
  eprintln!("Error: {}", msg);
  process::exit(1)
}

struct Rgb(u8, u8, u8);

fn rgbto8(c: &Rgb) -> u8 {
  (16 + (c.0 as u16)*6/256*36 + (c.1 as u16)*6/256*6 + (c.2 as u16)*6/256) as u8
}

// TODO: more flags
fn flag2colours(flag: &str) -> Vec<Rgb> {
  match flag {
    "pride" => vec![Rgb(255, 0, 0), Rgb(255, 165, 0), Rgb(255, 255, 0),
                    Rgb(0, 128, 0), Rgb(0, 0, 255), Rgb(128, 0, 128)],
    _ => oops!("unknown flag: {}", flag)
  }
}

fn colours(flags: &Vec<String>) -> Vec<Rgb> {
  flags.iter().flat_map(|f| flag2colours(f)).collect()
}

fn detect_truecolor() -> bool {
  let ct = env::var("COLORTERM").unwrap_or("".to_string());
  ct.contains("truecolor") || ct.contains("24bit")
}

fn setcolour(truecolor: bool, bg: bool, c: &Rgb) -> String {
  format!("\x1b[{};{}m", if bg { 48 } else { 38 }, if truecolor {
    format!("2;{};{};{}", c.0, c.1, c.2)
  } else {
    format!("5;{}", rgbto8(c))
  })
}

fn resetcolour(bg: bool) -> String {
  format!("\x1b[{};m", if bg { 49 } else { 39 })
}

struct Options { flags: Vec<String>, bg: bool, tc: bool }

fn parse_args() -> (Options, Vec<String>) {                   //  {{{1
  let mut flags = Vec::new();
  let mut bg    = false;
  let mut tc    = detect_truecolor();
  let mut files = Vec::new();
  let mut done  = false;
  let mut flag  = None;
  for arg in env::args().skip(1).collect::<Vec<String>>() {
    if flag.is_some() {
      flags.push(arg);
      flag = None
    } else if done {
      files.push(arg)
    } else {
      match arg.as_str() {
        "--"                    => done = true,
        "-f" | "--flag"         => flag = Some(arg),
        "-b" | "--background"   => bg   = true,
        "-t" | "--truecolor"    => tc   = true,
        "-T" | "--no-truecolor" => tc   = false,
        "--version" => {
          println!("proudcat-rust, version {}", VERSION);
          process::exit(0)
        }
        "--help" => {
          println!("{}", HELP);
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
  (Options { flags, bg, tc }, files)
}                                                             //  }}}1

fn main() {
  let stdin         = io::stdin();
  let (opts, files) = parse_args();
  let clrs          = colours(&opts.flags);
  let mut it        = clrs.iter().cycle();
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
      if sline.is_empty() && !opts.bg {
        print!("{}", line)
      } else {
        let sc = setcolour(opts.tc, opts.bg, it.next().unwrap());
        let rc = resetcolour(opts.bg);
        if opts.bg {
          print!("{}\x1b[2K{}{}", sc, line, rc)
        } else {
          let i = line.find(sline.chars().next().unwrap()).unwrap();
          print!("{}{}{}{}{}", line[..i].to_string(), sc, sline, rc,
                               line[i+sline.len()..].to_string())
        }
      }
      line.clear()
    }
  }
}

// vim: set tw=70 sw=2 sts=2 et fdm=marker :
