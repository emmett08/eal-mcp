//! Source-compatible native MAS codec command and persistent framing.
fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(eal_coordination::protocol::codec::entrypoint() as u8)
}
