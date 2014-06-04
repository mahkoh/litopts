#![crate_id="litopts_mac"]
#![crate_type="dylib"]
#![feature(managed_boxes, phase, macro_registrar, quote, macro_rules)]

extern crate syntax;
extern crate debug;
extern crate litopts;

use litopts::{LitOptFlag, LitOptOpt, LitOptOptOpt, OptType};

use std::string::{String};

use syntax::{ast};
use syntax::ast::{Name, TokenTree, LitStr, Expr, ExprVec, ExprLit};
use syntax::codemap::{Span, Pos};
use syntax::ext::base::{SyntaxExtension, DummyResult, ExtCtxt, MacResult, MacExpr,
                        NormalTT, BasicMacroExpander};
use syntax::parse::{token, new_parser_from_tts};
use syntax::parse::token::{InternedString, COMMA, EOF};

#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
    let bme = box BasicMacroExpander { expander: expand_opts, span: None };
    register(token::intern("litopts"), NormalTT(bme, None));
}

fn parse_macro(cx: &mut ExtCtxt,
               tts: &[TokenTree]) -> Option<Vec<(InternedString, Span)>> {
    let mut parser = new_parser_from_tts(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
    let mut bad = false;
    let mut opts = Vec::new();

    while parser.token != EOF {
        let row = cx.expand_expr(parser.parse_expr());
        let row_str = match row.node {
            ExprLit(lit) => match lit.node {
                LitStr(ref s, _) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        };
        match row_str {
            Some(s) => opts.push((s, row.span)),
            None => {
                bad = true;
                cx.span_err(row.span, "expected string literal");
            }
        }
        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    match bad {
        true => None,
        false => Some(opts),
    }
}

struct PreOpt {
    short: Option<char>,
    long: Option<String>,
    ty: OptType,
}

fn parse_opt(cx: &mut ExtCtxt, opt: &str, mut span: Span) -> Option<PreOpt> {
    macro_rules! err {
        ($i:expr, $m:expr) => {
            {
                span.lo = Pos::from_uint(span.lo.to_uint() + $i + 1);
                span.hi = span.lo;
                cx.span_err(span, $m);
                return None;
            }
        }
    };

    enum State {
        SStart,
        SDash,
        SShort,
        SShortOptOpt,
        SPostShort,
        SShortOpt,
        SDashDash,
        SLongOpt,
        SLongOptOpt,
        SEnd,
    }
    let mut state = SStart;
    let mut short = None;
    let mut long_start = None;
    let mut long_end = None;
    let mut ty = LitOptFlag;
    let mut pos = range(0, opt.len());
    macro_rules! consume {
        () => {
            match pos.next() {
                Some(i) => if opt[i] < 128 {
                    opt[i] as char
                } else {
                    err!(i, r"expected Ascii");
                },
                None => '☺',
            }
        }
    };
    loop {
        let (i, c) = match pos.next() {
            Some(i) => if opt[i] < 128 {
                (i, opt[i] as char)
            } else {
                err!(i, r"expected Ascii");
            },
            None    => (opt.len(), '☺'),
        };
        match state {
            SStart => {
                match c {
                    ' ' | '\t' => { },
                    '-' => state = SDash,
                    _ => err!(i, r"expected `-`"),
                }
            },
            SDash => {
                match c {
                    '-' => {
                        match consume!() {
                            'A'..'Z' | 'a'..'z' => { },
                            _ => err!(i+1, r"expected `[A-Za-z]`"),
                        }
                        long_start = Some(i+1);
                        state = SDashDash;
                    },
                    'A'..'Z' | 'a'..'z' => {
                        if short.is_some() {
                            err!(i, r"expected `-`");
                        }
                        short = Some(c);
                        state = SShort;
                    },
                    _ => err!(i, r"expected `[A-Za-z-]`"),
                }
            },
            SShort => {
                match c {
                    ' ' | '\t' => state = SPostShort,
                    '[' => {
                        ty = LitOptOptOpt;
                        state = SShortOptOpt;
                    },
                    ',' => state = SStart,
                    '☺' => break,
                    _ => err!(i, r"expected `[ \t\[]`"),
                }
            },
            SShortOptOpt => {
                match c {
                    'A'..'Z' | 'a'..'z' | '_' => { },
                    ']' => state = SEnd,
                    _ => err!(i, r"expected `[A-Za-z_\]]`"),
                }
            },
            SPostShort => {
                match c {
                    ' ' | '\t' => { },
                    '<' => {
                        ty = LitOptOpt;
                        state = SShortOpt;
                    },
                    ',' => state = SStart,
                    '☺' => break,
                    _ => err!(i, r"expected `[ \t<,]`"),
                }
            },
            SShortOpt => {
                match c {
                    'A'..'Z' | 'a'..'z' | '_' => { },
                    '>' => state = SEnd,
                    _ => err!(i, r"expected `[A-Za-z_>]`"),
                }
            },
            SDashDash => {
                match c {
                    'A'..'Z' | 'a'..'z' | '-' => { },
                    ' ' | '\t' => {
                        long_end = Some(i);
                        state = SEnd;
                    },
                    '=' => {
                        match consume!() {
                            'A'..'Z' | 'a'..'z' | '_' => { },
                            _ => err!(i+1, r"expected `[A-Za-z_]`"),
                        }
                        long_end = Some(i);
                        ty = LitOptOpt;
                        state = SLongOpt;
                    },
                    '[' => {
                        match consume!() {
                            '=' => { },
                            _ => err!(i+1, r"expected `=`"),
                        }
                        long_end = Some(i);
                        ty = LitOptOptOpt;
                        state = SLongOptOpt;
                    },
                    '☺' => {
                        long_end = Some(i);
                        break;
                    },
                    _ => err!(i, r"expected `[A-Za-z- \t=\[]`"),
                }
            },
            SLongOpt => {
                match c {
                    'A'..'Z' | 'a'..'z' | '_' => { },
                    ' ' | '\t' => state = SEnd,
                    '☺' => break,
                    _ => err!(i, r"expected `[A-Za-z_ \t]`"),
                }
            },
            SLongOptOpt => {
                match c {
                    'A'..'Z' | 'a'..'z' | '_' => { },
                    ']' => state = SEnd,
                    _ => err!(i, r"expected `[A-Za-z_\]]`"),
                }
            },
            SEnd => {
                match c {
                    ' ' | '\t' => { },
                    '☺' => break,
                    _ => err!(i, r"expected EOF"),
                }
            },
        }
    }

    let long = match long_start {
        Some(s) => Some(opt.slice(s, long_end.unwrap()).to_string()),
        None => None,
    };
    Some(PreOpt {
        short: short,
        long: long,
        ty: ty,
    })
}

fn expand_opts(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult> {
    let opts = match parse_macro(cx, tts) {
        Some(opts) => opts,
        None => return DummyResult::expr(sp),
    };
    let mut res = Vec::<PreOpt>::new();
    let mut bad = false;
    for &(ref opt_str, opt_span) in opts.iter() {
        match parse_opt(cx, opt_str.get(), opt_span) {
            Some(o) => {
                if o.short.is_some() && res.iter().any(|u| u.short == o.short) {
                    bad = true;
                    let c = o.short.unwrap();
                    let s = format!("duplicate flag `-{}`", c);
                    cx.span_err(opt_span, s.as_slice());
                } else if o.long.is_some() && res.iter().any(|u| u.long == o.long) {
                    bad = true;
                    let s = o.long.as_ref().unwrap();
                    let s = format!("duplicate flag `--{}`", s.as_slice());
                    cx.span_err(opt_span, s.as_slice());
                } else {
                    res.push(o);
                }
            },
            None => { }
        }
    }
    if bad {
        return DummyResult::expr(sp);
    }

    let cx = &*cx;
    let mut opts = Vec::new();
    for opt in res.iter() {
        let long = match opt.long {
            Some(ref v) => {
                let v = v.as_slice();
                quote_expr!(cx, Some($v))
            },
            _ => quote_expr!(cx, None)
        };
        let (short, short_str) = match opt.short {
            Some(s) => {
                let ss = format!("{}", s);
                let ss = ss.as_slice();
                (quote_expr!(cx, Some($s)), quote_expr!(cx, $ss))
            },
            _ => (quote_expr!(cx, None), quote_expr!(cx, "")),
        };
        let ty = match opt.ty {
            LitOptFlag   => quote_expr!(cx, ::litopts::LitOptFlag),
            LitOptOpt    => quote_expr!(cx, ::litopts::LitOptOpt),
            LitOptOptOpt => quote_expr!(cx, ::litopts::LitOptOptOpt),
        };
        opts.push(quote_expr!(cx, ::litopts::Opt { short:$short, short_str:$short_str,
                                                   long:$long, ty:$ty }));
    }
    let opts = @Expr { id: ast::DUMMY_NODE_ID, node: ExprVec(opts), span: sp };
    MacExpr::new(quote_expr!(cx, ::litopts::Opts { opts: $opts }))
}
