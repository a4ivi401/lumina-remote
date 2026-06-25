use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

const SERVICE_TYPE: &str = "_lumina._udp.local.";

/// Advertises this peer on the local network (LAN) via mDNS.
/// `port` is the local UDP port bound by Quinn.
/// `session_id` is the derived 12-char identity (used as the instance name).
pub fn advertise_local_service(port: u16, session_id: &str) -> Result<ServiceDaemon, String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

    let instance_name = session_id;
    let host_name = format!("{}.local.", session_id);

    let properties = HashMap::<String, String>::new();
    let my_service = ServiceInfo::new(
        SERVICE_TYPE,
        instance_name,
        &host_name,
        "", // auto-fills IP
        port,
        Some(properties),
    )
    .map_err(|e| format!("Failed to create mDNS ServiceInfo: {}", e))?;

    mdns.register(my_service)
        .map_err(|e| format!("Failed to register mDNS service: {}", e))?;

    Ok(mdns)
}

/// Discovers a specific host on the LAN by its session ID.
/// Returns the local `SocketAddr` of the target host.
pub async fn discover_local_host(
    session_id: &str,
    timeout_secs: u64,
) -> Result<SocketAddr, String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .map_err(|e| format!("Failed to browse mDNS: {}", e))?;

    let target_instance = format!("{}.{}", session_id, SERVICE_TYPE);

    let discovery_task = async {
        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname() == target_instance {
                    if let Some(ip) = info.get_addresses().iter().find(|ip| ip.is_ipv4()) {
                        let ip_str = ip.to_string();
                        let clean_ip = ip_str.split('%').next().unwrap_or(&ip_str);
                        if let Ok(parsed_ip) = clean_ip.parse::<std::net::IpAddr>() {
                            return Ok(SocketAddr::new(parsed_ip, info.get_port()));
                        }
                    }
                }
            }
        }
        Err::<SocketAddr, String>("mDNS receiver closed".to_string())
    };

    match timeout(Duration::from_secs(timeout_secs), discovery_task).await {
        Ok(Ok(addr)) => Ok(addr),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("mDNS LAN discovery timed out".to_string()),
    }
}

/// Discovers all Lumina hosts on the local network.
pub async fn discover_all_local_hosts(timeout_secs: u64) -> Result<Vec<String>, String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .map_err(|e| format!("Failed to browse mDNS: {}", e))?;

    let mut found_devices = Vec::new();

    let discovery_task = async {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let full_name = info.get_fullname();
                    // Name is usually LMN-1234-5678._lumina._udp.local.
                    if let Some(id) = full_name.split('.').next() {
                        if !found_devices.contains(&id.to_string()) {
                            found_devices.push(id.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    };

    let _ = timeout(Duration::from_secs(timeout_secs), discovery_task).await;
    
    Ok(found_devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Flaky on loopback interfaces
    async fn test_mdns_advertise_and_discover() {
        let session_id = "TEST-MDNS-1234";
        // Start advertiser on a random port
        let _daemon = advertise_local_service(55555, session_id).expect("Failed to advertise");

        // Try to discover it (with 5 seconds timeout)
        let addr = discover_local_host(session_id, 5).await;
        
        assert!(addr.is_ok(), "Failed to discover advertised mDNS service: {:?}", addr.err());
        let addr = addr.unwrap();
        assert_eq!(addr.port(), 55555);
    }
}
