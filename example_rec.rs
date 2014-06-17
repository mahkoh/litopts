#![feature(phase)]

#[phase(plugin)]
extern crate litopts_mac;
extern crate litopts;

use litopts::{OptUnknown};

#[deriving(PartialEq, Eq)]
enum ColorMode {
    /// Never write colored text.
    Never,
    /// Always write colored text.
    Always,
    /// Write colored text if the output is a terminal.
    Auto,
}

fn main() {
    static OPTS: litopts::Opts = litopts! {
        /// set color mode
        "-c, --color[=WHEN]",
        /// activate short mode
        "-s, --short",
        /// activate long mode
        "-l, --long",
        /// print this help
        "    --help",
        /// print the version
        "    --version",
    };

    let mut color_mode = Never;
    let mut short_mode = true;

    let args = std::os::args_as_bytes();
    let rec = match OPTS.record(args.tail()) {
        Ok(r) => r,
        Err(o) => match o.var {
            OptUnknown(o) => {
                println!("Unknown option {}", o);
                return;
            },
            _ => unreachable!(),
        },
    };
    for o in rec.res.iter() {
        match o.as_str {
            "s" => short_mode = true,
            "l" => short_mode = false,
            "c" => {
                match o.var.get_val_opt() {
                    Some(v) => match std::str::from_utf8(v) {
                        Some("never")  => color_mode = Never,
                        Some("always") => color_mode = Always,
                        Some("auto")   => color_mode = Auto,
                        _ => {
                            // o.real contains the string the option was activated with.
                            (writeln!(std::io::stderr(),
                                "Argument `{0}` takes no argument or one of the \
                                 arguments `never`, `always`, or `auto`.",
                                 o.real)).unwrap();
                            std::os::set_exit_status(1);
                            return;
                        }
                    },
                    None => color_mode = Always,
                }
            },
            "help" => {
                println!("USAGE:");
                print!("{}", OPTS.gahnoo_help());
                return;
            },
            "version" => {
                println!("1.0.0");
                return;
            },
            _ => unreachable!(),
        }
    }

    let mut stdout = std::io::stdio::stdout_raw();
    let colorize = (color_mode == Always) || (color_mode == Auto && stdout.isatty());

    for &f in rec.free.iter() {
        if short_mode {
            stdout.write_str("o: ").unwrap();
        } else {
            stdout.write_str("output: ").unwrap();
        }
        if colorize {
            stdout.write(bytes!("\x1b[33;1m")).unwrap();
        }
        stdout.write(f).unwrap();
        if colorize {
            stdout.write(bytes!("\x1b[0m")).unwrap();
        }
        stdout.write_str("\n").unwrap();
    }
}
