use std::num::ParseIntError;
use anyhow::bail;
use local_ip_address::{list_afinet_netifas};
use crate::const_value::{INDEX_OFFSET,  NET_INTERFACE_NAME};

pub fn get_local_ip() -> Option<String> {
    match list_afinet_netifas() {
        Ok(network_interfaces) => {
            println!("find ip:");
            for (name, ip) in network_interfaces.iter() {
                println!("  {}:\t{}", name, ip);
                if name == NET_INTERFACE_NAME && ip.is_ipv4() {
                    return Some(ip.to_string());
                }
            }
        }
        Err(e) => {
            eprintln!("get ip error: {:?}", e);
        }
    }
    None
}

pub fn splite_ip(ip_str: &str) -> Result<Vec<u8>, ParseIntError> {
    let parts: Result<Vec<u8>, _> = ip_str
        .split('.')
        .map(|s| s.parse::<u8>())
        .collect();

    parts
}

pub fn get_index() -> anyhow::Result<u8> {
    let ip = get_local_ip().unwrap();
    let ip_vec = splite_ip(&ip).unwrap();
    let last_number = *ip_vec.get(3).unwrap();
    if (last_number < 200) {
        bail!("last ip address is less than 200 ");
    }
    Ok(last_number - INDEX_OFFSET)
}