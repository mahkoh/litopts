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
    pub para: &'static str,
    pub help: &'static str,
    pub ty: OptType,
}

impl Opt {
    fn gahnoo_format(&self) -> String {
        let mut res = String::new();
        res.push_str("  ");
        if self.short.is_some() {
            res.push_char('-');
            res.push_char(self.short.unwrap());
            if self.long.is_some() {
                res.push_str(", ");
            } else {
                match self.ty {
                    LitOptOpt => res.push_str(format!(" <{}>", self.para).as_slice()),
                    LitOptOptOpt => res.push_str(format!("[{}]", self.para).as_slice()),
                    _ => { }
                }
            }
        }
        if self.long.is_some() {
            res.push_str("--");
            res.push_str(self.long.unwrap());
            match self.ty {
                LitOptOpt => res.push_str(format!("={}", self.para).as_slice()),
                LitOptOptOpt => res.push_str(format!("[={}]", self.para).as_slice()),
                _ => { }
            }
        }
        res
    }
}

pub struct Opts {
    pub opts: &'static [Opt],
}

pub struct Recording<'a> {
    pub free: Vec<&'a [u8]>,
    pub res: Vec<OptRes<'a>>,
}

impl<'a> Opts {
    pub fn getopts(&'a self, args: &'a [Vec<u8>]) -> OptsIter<'a> {
        OptsIter {
            opts: self,
            args: args,
            pos: 0,
            subpos: None,
            only_free: false,
            posix: false, 
        }
    }

    pub fn record(&'a self, args: &'a [Vec<u8>]) -> Result<Recording<'a>, OptRes<'a>> {
        let mut free = Vec::new();
        let mut res = Vec::new();
        for o in self.getopts(args) {
            match o.var {
                OptMissing(_) | OptLongMissing(_) | OptUnknown(_) => return Err(o),
                OptFree(v) => free.push(v),
                _ => res.push(o),
            }
        }
        Ok(Recording { free: free, res: res })
    }

    pub fn gahnoo_help(&'a self) -> String {
        let fmt: Vec<String> = self.opts.iter().map(|o| o.gahnoo_format()).collect();
        let has_both = self.opts.iter().any(|o| o.long.is_some() && o.short.is_some());
        let max_len = self.opts.iter().zip(fmt.iter()).map(|(o, f)| {
            if has_both && !o.short.is_some() {
                f.len() + 4
            } else {
                f.len()
            }
        }).max().unwrap_or(0);
        let offset = if max_len + 3 > 29 {
            29
        } else {
            max_len + 3
        };
        let mut res = String::new();
        for (o, f) in self.opts.iter().zip(fmt.iter()) {
            let mut real_len = f.len();
            if has_both && o.short.is_none() {
                res.push_str("    ");
                real_len += 4;
            }
            res.push_str(f.as_slice());
            let mut pos = if offset - real_len > 1 {
                res.push_str(" ".repeat(offset-real_len).as_slice());
                offset
            } else {
                res.push_char('\n');
                res.push_str(" ".repeat(offset+2).as_slice());
                offset + 2
            };
            let mut iter = o.help.words().peekable();
            loop {
                let word = match iter.next() {
                    Some(w) => w,
                    None => {
                        res.push_char('\n');
                        break;
                    },
                };
                if pos + word.len() > 80 {
                    pos = offset+2;
                    if pos + word.len() > 80 {
                        res.push_str(word);
                        if iter.peek().is_some() {
                            res.push_char('\n');
                            res.push_str(" ".repeat(offset+2).as_slice());
                        }
                        continue;
                    }
                    res.push_char('\n');
                    res.push_str(" ".repeat(offset+2).as_slice());
                }
                res.push_str(word);
                pos += word.len();
                if iter.peek().is_some() {
                    if pos < 80 {
                        res.push_char(' ');
                        pos += 1;
                    } else {
                        res.push_char('\n');
                        res.push_str(" ".repeat(offset+2).as_slice());
                        pos = offset + 2;
                    }
                }
            }
        }
        res
    }
}

pub struct OptRes<'a> {
    pub real: &'static str,
    pub as_str: &'static str,
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

impl<'a> OptVar<'a> {
    pub fn get_val(&self) -> &'a [u8] {
        match *self {
            OptOpt(_, v) => v,
            OptLongOpt(_, v) => v,
            _ => fail!(),
        }
    }

    pub fn get_val_opt(&self) -> Option<&'a [u8]> {
        match *self {
            OptOptOpt(_, v) => v,
            OptLongOptOpt(_, v) => v,
            _ => fail!(),
        }
    }
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
                            return Some(OptRes { real: o.short_str,
                                                 as_str: o.short_str,
                                                 var: $ex });
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
                    return Some(OptRes { real: "", as_str: "", var: OptUnknown(arg) });
                },
            }
        }
        let arg = &self.args[self.pos];
        if self.only_free || arg.len() < 2 || *arg.get(0) != '-' as u8 {
            self.pos += 1;
            if self.posix {
                self.only_free = true;
            }
            return Some(OptRes { real: "", as_str: "", var: OptFree(arg.as_slice()) });
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
                        ($as_str:expr, $ex:expr) => {
                            return Some(OptRes { real: o.long.unwrap(),
                                                 as_str: $as_str,
                                                 var: $ex });
                        }
                    };
                    self.pos += 1;
                    match o.ty {
                        LitOptFlag => {
                            if o.short.is_some() {
                                ret!(o.short_str, OptFlag(o.short.unwrap()));
                            }
                            ret!(o.long.unwrap(), OptLongFlag(o.long.unwrap()));
                        },
                        LitOptOpt => {
                            if p.is_some() {
                                let val = arg.slice(p.unwrap()+1, arg.len());
                                if o.short.is_some() {
                                    ret!(o.short_str,
                                         OptOpt(o.short.unwrap(), val));
                                }
                                ret!(o.long.unwrap(), OptLongOpt(o.long.unwrap(), val));
                            }
                            if self.pos < self.args.len() {
                                self.pos += 1;
                                let val = self.args[self.pos - 1].as_slice();
                                if o.short.is_some() {
                                    ret!(o.short_str, OptOpt(o.short.unwrap(), val));
                                }
                                ret!(o.long.unwrap(), OptLongOpt(o.long.unwrap(), val));
                            }
                            if o.short.is_some() {
                                ret!(o.short_str, OptMissing(o.short.unwrap()));
                            }
                            ret!(o.long.unwrap(), OptLongMissing(o.long.unwrap()));
                        },
                        LitOptOptOpt => {
                            if p.is_some() {
                                let val = arg.slice(p.unwrap()+1, arg.len());
                                if o.short.is_some() {
                                    ret!(o.short_str,
                                         OptOptOpt(o.short.unwrap(), Some(val)));
                                }
                                ret!(o.long.unwrap(),
                                     OptLongOptOpt(o.long.unwrap(), Some(val)));
                            }
                            if o.short.is_some() {
                                ret!(o.short_str, OptOptOpt(o.short.unwrap(), None));
                            }
                            ret!(o.long.unwrap(), OptLongOptOpt(o.long.unwrap(), None));
                        },
                    }
                },
                None => {
                    self.pos += 1;
                    if self.posix {
                        self.only_free = true;
                    }
                    return Some(OptRes { real: "", as_str: "",
                                         var: OptFree(arg.as_slice()) });
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
        Some(OptRes { real: "", as_str: "", var: OptFree(arg.as_slice()) })
    }
}
