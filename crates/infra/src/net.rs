//! This Mac's network addresses (P68: the Host Name the camera uploads to).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The addresses of every interface that is up, not loopback and not IPv6 link-local
/// (`getifaddrs`). Empty when there is no network.
pub fn local_addresses() -> Vec<IpAddr> {
    let mut found = Vec::new();
    let mut head: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: `getifaddrs` fills `head` with a list this function frees with `freeifaddrs`;
    // every node and its `ifa_addr` are read only while the list is alive.
    unsafe {
        if libc::getifaddrs(&mut head) != 0 {
            return found;
        }
        let mut node = head;
        while !node.is_null() {
            let entry = &*node;
            node = entry.ifa_next;
            let up = entry.ifa_flags & (libc::IFF_UP as u32) != 0;
            let loopback = entry.ifa_flags & (libc::IFF_LOOPBACK as u32) != 0;
            if !up || loopback || entry.ifa_addr.is_null() {
                continue;
            }
            let address = match i32::from((*entry.ifa_addr).sa_family) {
                libc::AF_INET => {
                    let sin = &*(entry.ifa_addr as *const libc::sockaddr_in);
                    IpAddr::V4(Ipv4Addr::from(u32::from_be(sin.sin_addr.s_addr)))
                }
                libc::AF_INET6 => {
                    let sin6 = &*(entry.ifa_addr as *const libc::sockaddr_in6);
                    let v6 = Ipv6Addr::from(sin6.sin6_addr.s6_addr);
                    if v6.is_unicast_link_local() {
                        continue;
                    }
                    IpAddr::V6(v6)
                }
                _ => continue,
            };
            if !found.contains(&address) {
                found.push(address);
            }
        }
        libc::freeifaddrs(head);
    }
    found
}
