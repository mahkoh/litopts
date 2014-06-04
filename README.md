## litopts

Command line option parsing.

### Short description

litopts allows you to specify possible command line options in a convenient way,
parse arbitrary posix style (-a) and --gnu-style options, and iterate over them
in the order the appeared in the command line.

### Details

The following example contains all possible option forms (modulo whitespace and
renaming):
```rust
static OPTS: libopts::Opts = litopts! {
    "-a",
    "--bbbb",
    "-c, --cccc",
    "-d <ARG>",
    "--eeee=ARG",
    "-f, --ffff=ARG",
    "-g[ARG]",
    "--hhhh[=ARG]",
    "-i, --iiii[=ARG]",
};
```
In the stream of command line options, these translate to the follwing variants:

Short | Long
----- | ---
`OptFlag('a')` | 
             | `OptLongFlag("bbbb")`
`OptFlag('c')` |
`OptOpt('d', v)` |
               | `OptLongOpt("eeee", v)`
`OptOpt('f', v)` |
`OptOptOpt('g', v)` |
                  | `OptLongOptOpt("hhhh", v)`
`OptOptOpt('i', v)` |

Note that litopts always chooses the shorter variant if possible.

Other possible variants are

Variant | Description
---|---
`OptMissing(c)` | Missing argument to a short option.
`OptLongMissing(s)` | Missing argument to a long option.
`OptUnknown(c)` | Unknown flag in a series of flags, e.g., in the example above consider the the argument `-acx`. This would triger `OptFlag('a')`, `OptFlag('c')`, and `OptUnknown('x')`.

In order to give helpful error messages, each parsed option in the stream comes
with the name that was actually used for it in the command line. E.g., in the
example above, `--cccc` trigers `OptFlag('c')` but it comes with a field
containing "cccc". See the example for details.

### Example

See `example.rs` for the code.

This program parses its arguments and writes each free argument to the command
line prefiex with `o: ` or `output: ` depending on the activated mode.
If colorization is enabled, it will print the argument in yellow.
