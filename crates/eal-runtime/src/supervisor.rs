//! Executable supervisor boundary. FD2 retains the inherited acquisition lease.
//! It is deliberately never used as a diagnostic stream.
#[cfg(unix)]
mod unix;

/// Entry point for the installed eal-command-supervisor companion binary.
pub fn entrypoint() -> i32 {
    #[cfg(unix)]
    {
        unix::entrypoint()
    }
    #[cfg(not(unix))]
    {
        2
    }
}
