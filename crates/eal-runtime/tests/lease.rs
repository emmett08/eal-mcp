#![cfg(unix)]
use eal_runtime::lease::WorkspaceAcquisition;
use std::{
    fs,
    os::{
        fd::AsFd,
        unix::fs::{DirBuilderExt, PermissionsExt},
    },
    path::PathBuf,
    time::{Duration, Instant},
};
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-lease-test-{}", uuid::Uuid::new_v4()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn descriptor_clone_keeps_the_lease_after_the_original_guard_is_dropped() {
    let directory = Directory::new();
    let coordination = WorkspaceAcquisition::new(&directory.0).unwrap();
    let lease = coordination.hold(Duration::ZERO).unwrap();
    let inherited = lease.as_fd().try_clone_to_owned().unwrap();
    drop(lease);
    assert!(coordination.hold(Duration::ZERO).is_err());
    drop(inherited);
    assert!(coordination.hold(Duration::ZERO).is_ok());
}
#[test]
fn unwinding_closes_only_the_parent_copy() {
    let directory = Directory::new();
    let coordination = WorkspaceAcquisition::new(&directory.0).unwrap();
    let mut inherited = None;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let lease = coordination.hold(Duration::ZERO).unwrap();
            inherited = Some(lease.as_fd().try_clone_to_owned().unwrap());
            panic!("Synthetic acquisition interruption");
        }))
        .is_err()
    );
    assert!(coordination.hold(Duration::ZERO).is_err());
    drop(inherited);
    assert!(coordination.hold(Duration::ZERO).is_ok());
}
#[test]
fn competing_threads_observe_a_bounded_timeout_and_can_retry() {
    let directory = Directory::new();
    let coordination = WorkspaceAcquisition::new(&directory.0).unwrap();
    let lease = coordination.hold(Duration::ZERO).unwrap();
    let other = coordination.clone();
    let started = Instant::now();
    std::thread::spawn(move || {
        assert!(
            other
                .hold(Duration::from_millis(50))
                .unwrap_err()
                .contains("Timed out")
        )
    })
    .join()
    .unwrap();
    assert!(started.elapsed() < Duration::from_secs(1));
    drop(lease);
    assert!(coordination.hold(Duration::ZERO).is_ok());
}
#[test]
fn unsafe_lock_nodes_are_rejected_without_waiting() {
    for kind in ["fifo", "symlink", "public", "hardlink", "directory"] {
        let directory = Directory::new();
        let coordination = WorkspaceAcquisition::new(&directory.0).unwrap();
        let path = coordination.path();
        match kind {
            "fifo" => rustix::fs::mkfifoat(
                rustix::fs::CWD,
                path,
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .unwrap(),
            "directory" => fs::create_dir(path).unwrap(),
            "public" => {
                fs::write(path, []).unwrap();
                fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
            }
            _ => {
                let target = directory.0.join("target");
                fs::write(&target, []).unwrap();
                fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
                if kind == "symlink" {
                    std::os::unix::fs::symlink(&target, path).unwrap();
                } else {
                    fs::hard_link(&target, path).unwrap();
                }
            }
        }
        let started = Instant::now();
        assert!(coordination.hold(Duration::ZERO).is_err(), "{kind}");
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
#[test]
fn directory_replacement_and_public_permissions_are_rejected() {
    let directory = Directory::new();
    let coordination = WorkspaceAcquisition::new(&directory.0).unwrap();
    fs::set_permissions(directory.0.join(".eal"), fs::Permissions::from_mode(0o777)).unwrap();
    assert!(coordination.hold(Duration::ZERO).is_err());
    assert!(WorkspaceAcquisition::new(&directory.0).is_err());
}
