![Banner](https://s-christy.com/sbs/status-banner.svg?icon=action/terminal&hue=30&title=Coreutils-rs&description=GNU%20coreutils%20reimplemented%20in%20Rust)

## Overview

Coreutils-rs is a Rust reimplementation of the GNU coreutils - the foundational
command-line utilities found on virtually every Unix-like system. The project
is structured as a multi-call binary, meaning a single executable can be
invoked under different names (or with a subcommand argument) to behave as any
of the supported tools.

The implementation covers over 60 utilities spanning file operations, text
processing, checksums, user info, math, and more.

## Features

- Multi-call binary: invoke as `coreutils-rs <command>` or symlink the binary
- File and directory operations: `ls`, `cp`, `mv`, `rm`, `mkdir`, `rmdir`
- File viewing and text utilities: `cat`, `tac`, `head`, `tail`, `less`, `more`, `nl`, `wc`
- Text processing: `cut`, `paste`, `sort`, `uniq`, `tr`, `comm`, `join`
- Searching and comparing: `cmp`, `diff`
- Permissions and ownership: `chmod`, `chown`, `chgrp`
- Disk and filesystem: `df`, `du`, `stat`, `sync`
- Date and time: `date`, `sleep`, `timeout`
- Math and sequences: `expr`, `seq`, `factor`
- Symbolic and hard links: `ln`, `readlink`
- User and group info: `whoami`, `id`, `groups`, `users`
- Checksums: `md5sum`, `sha1sum`, `sha256sum`, `sha512sum`, `sum`, `cksum`
- Path manipulation: `basename`, `dirname`, `realpath`
- Miscellaneous: `echo`, `printf`, `yes`, `true`, `false`, `test`, `[`

## Usage

```
Usage: coreutils-rs <command> [args...]
```

The binary can also be invoked via a symlink named after the desired command:

```
ln -s coreutils-rs ls
./ls
```

## Dependencies

```
cargo
rustc
```

## License

This work is licensed under the GNU General Public License version 3 (GPLv3).

[<img src="https://s-christy.com/status-banner-service/GPLv3_Logo.svg" width="150" />](https://www.gnu.org/licenses/gpl-3.0.en.html)
