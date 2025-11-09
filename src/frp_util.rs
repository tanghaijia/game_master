use std::process::ExitStatus;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::process::Command;
use crate::const_value::{FRPC_EXE_PATH, FRPC_TOML_PATH, TCP_LOCAL_PORT, UDP_LOCAL_PORT};

#[derive(Serialize, Deserialize, Debug)]
pub struct FrpcToml {
    pub server_addr: String,
    pub server_port: u16,
    pub auth_token: String,
    pub tcp_name: String,
    pub tcp_remote_port: u16,
    pub udp_name: String,
    pub udp_remote_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Auth {
    token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WebServer {
    addr: String,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Proxy {
    name: String,
    #[serde(rename = "type")]
    proxy_type: String,
    #[serde(rename = "localIP")]
    local_ip: String,
    local_port: u16,
    remote_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    server_addr: String,
    server_port: u16,
    auth: Auth,
    web_server: WebServer,
    proxies: Vec<Proxy>,
}

pub async fn frpc_config_read(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path).await?;

    let config: Config = toml::from_str(&contents)?;

    println!("read frpc.toml:\n{:#?}", config);
    Ok(config)
}

pub async fn frpc_config_write(config: &FrpcToml, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let auth = Auth{ token: config.auth_token.clone() };
    let server_addr = config.server_addr.clone();
    let server_port = config.server_port;
    let tcp = Proxy {
        name: config.tcp_name.clone(),
        proxy_type: "tcp".to_string(),
        local_port: TCP_LOCAL_PORT,
        local_ip: "127.0.0.1".to_string(),
        remote_port: config.tcp_remote_port
    };
    let ucp = Proxy {
        name: config.udp_name.clone(),
        proxy_type: "udp".to_string(),
        local_port: UDP_LOCAL_PORT,
        local_ip: "127.0.0.1".to_string(),
        remote_port: config.udp_remote_port
    };
    let web_server = WebServer {
        addr: "127.0.0.1".to_string(),
        port: 7400
    };
    let frpc_config = Config {
        server_port,
        server_addr,
        auth,
        web_server,
        proxies: vec![tcp, ucp]
    };

    let toml_string = toml::to_string_pretty(&frpc_config)?;
    fs::write(path, toml_string).await?;

    println!("sucess write frpc.toml to {}", path);

    Ok(())
}

pub async fn frpc_config_reset_by_index(path: &str, index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path).await?;

    let mut config: Config = toml::from_str(&contents)?;
    for proxy in config.proxies.iter_mut() {
        if proxy.proxy_type == "tcp" {
            proxy.name = format!("7daysTodieServer-{}", index);
            proxy.remote_port = TCP_LOCAL_PORT + index as u16;
        } else if proxy.proxy_type == "udp" {
            proxy.name = format!("7daysTodieServerUDP-{}", index);
            proxy.remote_port = UDP_LOCAL_PORT + index as u16;
        }
    }

    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(path, toml_string).await?;

    frpc_config_reload().await?;

    Ok(())
}

pub async fn frpc_config_reload() -> anyhow::Result<ExitStatus> {
    let status = Command::new(FRPC_EXE_PATH)
        .arg("reload")
        .arg("-c")
        .arg(FRPC_TOML_PATH)
        .spawn()?
        .wait().await.unwrap();

    Ok(status)
}

#[cfg(test)]
mod tests {
    use crate::frp_util::{frpc_config_read, frpc_config_write, FrpcToml};

    #[tokio::test]
    async fn frpc_config_write_test() {
        let config = FrpcToml {
            server_addr: "124.223.27.133".to_string(),
            server_port: 7000,
            auth_token: "123456".to_string(),
            tcp_name: "7daysTodieServer".to_string(),
            tcp_remote_port: 26900,
            udp_name: "7daysTodieServerUDP26902".to_string(),
            udp_remote_port: 26902,
        };
        let res = frpc_config_write(&config, "C:\\Users\\89396\\projects\\game_master\\frpc.toml").await.unwrap();

        ()
    }

    #[tokio::test]
    async fn frpc_config_read_test() {
        let config = frpc_config_read("C:\\Users\\89396\\Downloads\\frp_0.65.0_windows_amd64\\frp_0.65.0_windows_amd64\\frpc.toml").await.unwrap();
    }
}
