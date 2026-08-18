use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub mac: String,
    pub vendor: String,
}

pub struct NetworkScanner;

impl NetworkScanner {
    pub async fn run_scan() -> Vec<DiscoveredDevice> {
        println!("🔍 [Scanner] Waking up sleeping WiFi and LAN devices...");
        let mut devices = Vec::new();

        // 1. إيجاد كارت الشبكة المحلي والـ IP الخاص بالجهاز (Host Machine)
        let mut host_ip = String::new();
        let mut host_mac = String::new();
        let mut subnet = "192.168.0.0/24".to_string();

        if let Ok(route_out) = Command::new("ip")
            .args(&["route", "get", "1.1.1.1"])
            .output()
            .await
        {
            let out_str = String::from_utf8_lossy(&route_out.stdout);
            let parts: Vec<&str> = out_str.split_whitespace().collect();

            if let Some(dev_idx) = parts.iter().position(|&r| r == "dev") {
                if let Some(iface) = parts.get(dev_idx + 1) {
                    let mac_path = format!("/sys/class/net/{}/address", iface);
                    if let Ok(mac) = std::fs::read_to_string(mac_path) {
                        host_mac = mac.trim().to_uppercase();
                    }
                    // نجلب النطاق (Subnet) الخاص بهذا الكارت
                    let awk_cmd =
                        format!("ip -o -f inet addr show dev {} | awk '{{print $4}}'", iface);
                    if let Ok(sub_out) = Command::new("sh").arg("-c").arg(&awk_cmd).output().await {
                        let parsed_subnet =
                            String::from_utf8_lossy(&sub_out.stdout).trim().to_string();
                        if !parsed_subnet.is_empty() {
                            subnet = parsed_subnet;
                        }
                    }
                }
            }
            if let Some(src_idx) = parts.iter().position(|&r| r == "src") {
                if let Some(ip) = parts.get(src_idx + 1) {
                    host_ip = ip.to_string();
                }
            }
        }

        // 2. إيقاظ الأجهزة (3 مرات كما في السكربت الخاص بك)
        let _ = Command::new("fping")
            .args(&["-c", "3", "-g", &subnet])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        // 3. مسح الأجهزة عبر arp-scan (بدون sudo لأن الديمون سيعمل كـ Root)
        let arp_output = Command::new("arp-scan")
            .args(&["--localnet", "--ignoredups"])
            .output()
            .await;

        if let Ok(output) = arp_output {
            let result = String::from_utf8_lossy(&output.stdout);

            for line in result.lines() {
                if !line.contains(":")
                    || line.starts_with("Interface")
                    || line.starts_with("Starting")
                    || line.starts_with("Ending")
                {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let ip = parts[0].to_string();
                    let mac = parts[1].to_uppercase();
                    let vendor = parts[2..].join(" ");
                    devices.push(DiscoveredDevice { ip, mac, vendor });
                }
            }
        } else {
            eprintln!("⚠️ [Scanner] Failed to run arp-scan. Ensure netwatchd is running as root.");
        }

        // 4. إضافة الجهاز المحلي (Host Machine) إلى قائمة الأجهزة المكتشفة!
        if !host_ip.is_empty() && !host_mac.is_empty() {
            devices.push(DiscoveredDevice {
                ip: host_ip,
                mac: host_mac,
                vendor: "Host Machine (This Computer)".to_string(),
            });
        }

        devices
    }
}
