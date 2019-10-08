# great-appender

Append string repeatedly to specified file.

Only after writing it did I understood that it was `yes(1)` in
disguise. Anyways, it's a small Rust program.

The thing of note is to use pre-filled buffer to avoid calling
`file.write` too often.

## Usage

```rust
$ great-appender -h
great-appender 1.0
Tim Marinin <mt@marinintim.com>
append some message to file repeatedly

USAGE:
    great-appender [FLAGS] [OPTIONS] --file <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    print

OPTIONS:
    -f, --file <file>              file to append message to
    -m, --message <message>        message to append [default: message]
    -p, --per_write <per_write>     [default: 1024]
```

## License

I hereby declare source code of great-appender (but not its dependencies, which
come with their own licensing terms) into public domain.
