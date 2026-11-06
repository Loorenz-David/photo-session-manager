use ipnet::IpNet;
use std::net::IpAddr;

pub fn is_allowed(ip: IpAddr, nets: &[IpNet]) -> bool {
    nets.iter().any(|net| net.contains(&ip))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allowed_networks() {
        let nets = crate::ftp::ReceiverConfig::default_allowed_peers();
        for ip in [
            "10.0.0.5",
            "172.16.0.1",
            "172.31.255.254",
            "192.168.1.20",
            "169.254.1.1",
            "fd00::1",
            "fe80::1",
        ] {
            assert!(is_allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
        for ip in [
            "8.8.8.8",
            "127.0.0.1",
            "172.32.0.1",
            "11.0.0.1",
            "::1",
            "2001:db8::1",
        ] {
            assert!(!is_allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
    }
}
