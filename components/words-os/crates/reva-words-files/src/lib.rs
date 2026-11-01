//! # reva-words-files — the file words, file system calls, directories
//!
//! The newcore file words (`open/r`, `open/rw`, `creat`, `close`, `read`,
//! `write`, `fsize`, `(save)`) keep their stack effects and their `ioerr`
//! rule: 0 on success, the negative `errno` on failure, as the original's
//! `call_linux_return` left it.  `reva.f`'s other file words are Linux
//! system calls (`seek`, `rename`, `delete`, `stat`, `chdir`, `getcwd`,
//! and `mkdir`/`rmdir` in `lib/os/dir`); [`sysfile::SysFile::sys_file`]
//! serves those numbers, and the virtual libc gains `opendir`/`readdir`/
//! `closedir` for `lib/os/dir` ([`dirs`]).

pub mod dirs;
pub mod filewords;
pub mod save;
pub mod sysfile;
