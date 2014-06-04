#![feature(phase)]

#[phase(syntax)]
extern crate litopts_mac;
extern crate litopts;

use litopts::{OptFlag, OptOptOpt, OptLongFlag, OptFree};

#[deriving(PartialEq, Eq)]
enum ColorMode {
    Never,
    Always,
    Auto,
}

fn main() {
    static OPTS: litopts::Opts = litopts! {
        "-c, --color[=WHEN]",
        "-s, --short",
        "-l, --long",
        "    --version",
    };

    let mut color_mode = Never;
    let mut short_mode = true;
    let mut free = Vec::new();

    let args = std::os::args_as_bytes();
    for o in OPTS.getopts(args.tail()) {
        match o.var {
            OptFlag('s') => short_mode = true,
            OptFlag('l') => short_mode = false,
            OptOptOpt('c', v) => {
                match v {
                    Some(v) => match std::str::from_utf8(v) {
                        Some("never")  => color_mode = Never,
                        Some("always") => color_mode = Always,
                        Some("auto")   => color_mode = Auto,
                        _ => {
                            (writeln!(std::io::stderr(),
                                "Argument `{0}` has to be of the form `{0}`, \
                                 `{0}=never`, `{0}=always`, or `{0}=auto`.",
                                 o.real)).unwrap();
                            std::os::set_exit_status(1);
                            return;
                        }
                    },
                    None => color_mode = Always,
                }
            },
            OptLongFlag("version") => {
                println!("1.0.0");
                return;
            },
            OptFree(v) => free.push(v),
            _ => unreachable!(),
        }
    }

    let mut stdout = std::io::stdio::stdout_raw();
    let colorize = (color_mode == Always) || (color_mode == Auto && stdout.isatty());

    for &f in free.iter() {
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
