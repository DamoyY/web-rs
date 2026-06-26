use super::is_address_in_use;
use std::io;
#[test]
fn address_in_use_error_is_downgradable() {
    let error = io::Error::from(io::ErrorKind::AddrInUse);
    assert!(is_address_in_use(&error));
}
#[test]
fn other_io_errors_remain_fatal() {
    let error = io::Error::from(io::ErrorKind::PermissionDenied);
    assert!(!is_address_in_use(&error));
}
