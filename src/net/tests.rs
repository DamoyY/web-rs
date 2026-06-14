use crate::net::ssrf::is_public_ip;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
#[test]
fn private_and_loopback_addresses_are_not_public() {
    assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)));
    assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
}
#[test]
fn documentation_public_example_address_is_allowed() {
    assert!(is_public_ip(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))));
}
#[test]
fn unique_local_ipv6_address_is_not_public() {
    assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::new(
        0xfc00, 0, 0, 0, 0, 0, 0, 1
    ))));
}
