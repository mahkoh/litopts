#![crate_id="litopts"]
#![crate_type="lib"]
#![feature(macro_rules)]

#[deriving(PartialEq, Eq)]
pub enum OptType {
    LitOptFlag,
    LitOptOpt,
    LitOptOptOpt,
}

pub struct Opt {
    pub short: Option<char>,
    pub short_str: &'static str,
    pub long: Option<&'static str>,
    pub ty: OptType,
}

pub struct Opts {
    pub opts: &'static [Opt],
}

impl Opts {
    pub fn getopts<'a>(&'a self, args: &'a [Vec<u8>]) -> OptsIter<'a> {
        OptsIter {
            opts: self,
            args: args,
            pos: 0,
            subpos: None,
            only_free: false,
            posix: false, 
        }
    }
}

pub struct OptRes<'a> {
    pub real: &'static str,
    pub var: OptVar<'a>,
}

pub enum OptVar<'a> {
    OptFlag(char),
    OptOpt(char, &'a [u8]),
    OptOptOpt(char, Option<&'a [u8]>),
    OptLongFlag(&'static str),
    OptLongOpt(&'static str, &'a [u8]),
    OptLongOptOpt(&'static str, Option<&'a [u8]>),
    OptFree(&'a [u8]),

    OptMissing(char),
    OptLongMissing(&'static str),
    OptUnknown(char),
}

pub struct OptsIter<'a> {
    opts: &'a Opts,
    args: &'a [Vec<u8>],
    pos: uint,
    subpos: Option<uint>,
    only_free: bool,
    pub posix: bool,
}

impl<'a> Iterator<OptRes<'a>> for OptsIter<'a> {
    fn next(&mut self) -> Option<OptRes<'a>> {
        match self.subpos {
            Some(p) => if p >= self.args[self.pos].len() {
                self.pos += 1;
                self.subpos = None;
            },
            None => { }
        }
        if self.pos >= self.args.len() {
            return None;
        }
        if self.subpos.is_some() {
            let subpos = self.subpos.unwrap();
            let arg = *self.args[self.pos].get(subpos) as char;
            match self.opts.opts.iter().find(|f| f.short == Some(arg)) {
                Some(o) => {
                    macro_rules! ret {
                        ($ex:expr) => {
                            return Some(OptRes { real: o.short_str, var: $ex });
                        }
                    };
                    if o.ty == LitOptFlag {
                        self.subpos = Some(subpos + 1);
                        ret!(OptFlag(arg));
                    }
                    self.subpos = None;
                    self.pos += 1;
                    if o.ty == LitOptOptOpt {
                        if subpos + 1 < self.args[self.pos - 1].len() {
                            let val = self.args[self.pos - 1].tailn(subpos + 1);
                            ret!(OptOptOpt(arg, Some(val)));
                        }
                        ret!(OptOptOpt(arg, None));
                    }
                    if subpos + 1 < self.args[self.pos - 1].len() {
                        let val = self.args[self.pos - 1].tailn(subpos + 1);
                        ret!(OptOpt(arg, val));
                    }
                    if self.pos < self.args.len() {
                        self.pos += 1;
                        let val = self.args[self.pos - 1].as_slice();
                        ret!(OptOpt(arg, val));
                    }
                    ret!(OptMissing(arg));
                },
                None => {
                    self.subpos = None;
                    self.pos += 1;
                    return Some(OptRes { real: "", var: OptUnknown(arg) });
                },
            }
        }
        let arg = &self.args[self.pos];
        if self.only_free || arg.len() < 2 || *arg.get(0) != '-' as u8 {
            self.pos += 1;
            if self.posix {
                self.only_free = true;
            }
            return Some(OptRes { real: "", var: OptFree(arg.as_slice()) });
        }
        if arg.len() >= 2 && *arg.get(1) == '-' as u8 {
            if arg.len() == 2 {
                self.pos += 1;
                self.only_free = true;
                return self.next();
            }
            let (arg_s, p) = match arg.iter().position(|&c| c == '=' as u8) {
                Some(p) => (arg.slice(2, p), Some(p)),
                None => (arg.tailn(2), None),
            };
            match self.opts.opts.iter().filter(|o| o.long.is_some())
                                       .find(|o| o.long.unwrap().as_bytes() == arg_s) {
                Some(o) => {
                    macro_rules! ret {
                        ($ex:expr) => {
                            return Some(OptRes { real: o.long.unwrap(), var: $ex });
                        }
                    };
                    self.pos += 1;
                    match o.ty {
                        LitOptFlag => {
                            if o.short.is_some() {
                                ret!(OptFlag(o.short.unwrap()));
                            }
                            ret!(OptLongFlag(o.long.unwrap()));
                        },
                        LitOptOpt => {
                            if p.is_some() {
                                let val = arg.slice(p.unwrap()+1, arg.len());
                                if o.short.is_some() {
                                    ret!(OptOpt(o.short.unwrap(), val));
                                }
                                ret!(OptLongOpt(o.long.unwrap(), val));
                            }
                            if self.pos < self.args.len() {
                                self.pos += 1;
                                let val = self.args[self.pos - 1].as_slice();
                                if o.short.is_some() {
                                    ret!(OptOpt(o.short.unwrap(), val));
                                }
                                ret!(OptLongOpt(o.long.unwrap(), val));
                            }
                            if o.short.is_some() {
                                ret!(OptMissing(o.short.unwrap()));
                            }
                            ret!(OptLongMissing(o.long.unwrap()));
                        },
                        LitOptOptOpt => {
                            if p.is_some() {
                                let val = arg.slice(p.unwrap()+1, arg.len());
                                if o.short.is_some() {
                                    ret!(OptOptOpt(o.short.unwrap(), Some(val)));
                                }
                                ret!(OptLongOptOpt(o.long.unwrap(), Some(val)));
                            }
                            if o.short.is_some() {
                                ret!(OptOptOpt(o.short.unwrap(), None));
                            }
                            ret!(OptLongOptOpt(o.long.unwrap(), None));
                        },
                    }
                },
                None => {
                    self.pos += 1;
                    if self.posix {
                        self.only_free = true;
                    }
                    return Some(OptRes { real: "", var: OptFree(arg.as_slice()) });
                },
            }
        }
        if self.opts.opts.iter().any(|o| o.short == Some(*arg.get(1) as char)) {
            self.subpos = Some(1);
            return self.next();
        }
        self.pos += 1;
        if self.posix {
            self.only_free = true;
        }
        Some(OptRes { real: "", var: OptFree(arg.as_slice()) })
    }
}
