use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

pub fn encode_header(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

pub fn decode_header(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().to_string()
}

pub fn lan_ips() -> Vec<String> {
    let mut entries: Vec<(String, Ipv4Addr)> = Vec::new();
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in interfaces {
            if let IpAddr::V4(v4) = ip {
                if v4.is_loopback() || v4.is_link_local() || v4.is_unspecified() || v4 == Ipv4Addr::BROADCAST {
                    continue;
                }
                if entries.iter().any(|(_, existing)| *existing == v4) {
                    continue;
                }
                entries.push((name.to_ascii_lowercase(), v4));
            }
        }
    }
    entries.sort_by_key(|(name, ip)| (rank_iface(name), rank_ip(*ip)));
    let ips: Vec<String> = entries.into_iter().map(|(_, ip)| ip.to_string()).collect();
    if ips.is_empty() {
        vec!["127.0.0.1".into()]
    } else {
        ips
    }
}

pub fn lan_ip() -> String {
    lan_ips().into_iter().next().unwrap_or_else(|| "127.0.0.1".into())
}

fn rank_ip(ip: Ipv4Addr) -> u8 {
    let octets = ip.octets();
    if octets[0] == 192 && octets[1] == 168 {
        0
    } else if octets[0] == 10 {
        1
    } else if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        2
    } else if ip.is_private() {
        3
    } else if octets[0] == 100 && (64..=127).contains(&octets[1]) {
        4
    } else {
        5
    }
}

fn rank_iface(name: &str) -> u8 {
    const VIRTUAL: &[&str] = &[
        "vethernet",
        "vmware",
        "virtualbox",
        "hyper-v",
        "loopback",
        "tailscale",
        "zerotier",
        "happ",
        "xray",
        "vpn",
        "wintun",
        "wireguard",
        "openvpn",
        "tap-",
        "tun",
        "docker",
        "wsl",
        "npcap",
        "radmin",
    ];
    if VIRTUAL.iter().any(|needle| name.contains(needle)) {
        2
    } else if name.contains("wi-fi") || name.contains("wifi") || name.contains("ethernet") {
        0
    } else {
        1
    }
}

pub fn content_disposition(name: &str, inline: bool) -> String {
    let kind = if inline { "inline" } else { "attachment" };
    let fallback: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '-' | '_' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!(
        "{kind}; filename=\"{fallback}\"; filename*=UTF-8''{}",
        encode_header(name)
    )
}

pub fn qr_svg(data: &str, pixels: u32) -> Option<String> {
    let code = qrcode::QrCode::new(data.as_bytes()).ok()?;
    Some(
        code.render::<qrcode::render::svg::Color>()
            .min_dimensions(pixels, pixels)
            .quiet_zone(true)
            .dark_color(qrcode::render::svg::Color("#14232D"))
            .light_color(qrcode::render::svg::Color("#FFFFFF"))
            .build(),
    )
}

pub fn open_path(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|e| e.to_string())
}

pub fn reveal_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return open::that(path).map_err(|e| e.to_string());
    }
    // Explorer needs the path quoted but the `/select,` switch outside the quotes:
    //   explorer.exe /select,"C:\path with spaces\file.txt"
    // Passing it as normal Command arguments makes Rust quote the entire switch,
    // which Explorer does not recognise and it falls back to Documents.
    use std::os::windows::process::CommandExt;
    let arg = format!("/select,\"{}\"", path.display());
    std::process::Command::new("explorer.exe")
        .raw_arg(arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
