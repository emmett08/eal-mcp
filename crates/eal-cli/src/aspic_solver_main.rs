//! One checked ASPIC request, with no runtime state or process supervision.
#![forbid(unsafe_code)]

mod aspic_solver;

fn main() -> std::process::ExitCode {
    aspic_solver::main()
}
