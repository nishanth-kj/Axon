pub fn normalize_ip(ip: &str) -> String {
    ip.trim()
        .strip_prefix("::ffff:")
        .unwrap_or(ip.trim())
        .to_string()
}

pub fn is_local_ip(ip: &str) -> bool {
    ip == "localhost"
        || ip == "127.0.0.1"
        || ip == "::1"
        || ip.starts_with("192.168.")
        || ip.starts_with("10.")
}

pub fn bracket_host(ip: &str) -> String {
    if ip.contains(':') {
        format!("[{ip}]")
    } else {
        ip.to_string()
    }
}

pub fn client_ip(
    peer: Option<std::net::SocketAddr>,
    forwarded_for: Option<&str>,
    real_ip: Option<&str>,
) -> String {
    if let Some(forwarded) = forwarded_for {
        if let Some(first) = forwarded.split(',').next() {
            let ip = normalize_ip(first);
            if !ip.is_empty() {
                return ip;
            }
        }
    }
    if let Some(real) = real_ip {
        let ip = normalize_ip(real);
        if !ip.is_empty() {
            return ip;
        }
    }
    peer.map(|addr| normalize_ip(&addr.ip().to_string()))
        .filter(|ip| !ip.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwarded_header_beats_the_socket_address() {
        let peer = "10.0.0.8:40000".parse().ok();
        let ip = client_ip(peer, Some("203.0.113.9, 10.0.0.1"), Some("10.0.0.1"));
        assert_eq!(ip, "203.0.113.9");
    }

    #[test]
    fn mapped_ipv4_is_unwrapped() {
        let mapped = client_ip(None, Some("::ffff:127.0.0.1"), None);
        assert_eq!(mapped, "127.0.0.1");
    }

    #[test]
    fn missing_headers_fall_back_to_unknown() {
        assert_eq!(client_ip(None, None, None), "unknown");
    }
}
