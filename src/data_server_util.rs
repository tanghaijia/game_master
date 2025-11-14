use crate::game_config_util::{ServerSettings};
use std::env;
use serde::{Deserialize, Serialize};
use crate::const_value::DATA_SERVER_PORT;

pub async fn get_game_config_by_serverconfig_id(serverconfig_id: i32) -> anyhow::Result<ServerSettings> {
    let data_server_ip_address = env::var("DATA_SERVER_IP_ADDR")?;
    let url = format!("http://{}:{}/api/game_master/game_config?serverconfig_id={}",
        data_server_ip_address, DATA_SERVER_PORT, serverconfig_id,);
    println!("get_game_config_by_serverconfig_id url: {}", url);
    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let dog_api_response = response.json::<ServerSettings>().await?;
        println!("get serverconfig: {:?}", dog_api_response);
        Ok(dog_api_response)
    } else {
        println!("请求失败，状态码: {}", response.status());
        Err(anyhow::anyhow!("请求失败，状态码: {}", response.status()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct  SaveFileInfo {
    pub id: i32,
    pub name: String,
    pub user_id: String,
    pub bucket_name: String,
    pub host: String,
    pub createdAt: String,
    pub updatedAt: String
}
pub async fn get_savefile_info_by_save_file_id(save_file_id: i32) -> anyhow::Result<SaveFileInfo> {
    let data_server_ip_address = env::var("DATA_SERVER_IP_ADDR")?;
    let url = format!("http://{}:{}/api/game_master/download_savefile?save_file_id={}",
                      data_server_ip_address, DATA_SERVER_PORT, save_file_id,);
    println!("get_savefile_info_by_save_file_id url: {}", url);
    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let dog_api_response = response.json::<SaveFileInfo>().await?;
        println!("get savefile info: {:?}", dog_api_response);
        Ok(dog_api_response)
    } else {
        println!("请求失败，状态码: {}", response.status());
        Err(anyhow::anyhow!("请求失败，状态码: {}", response.status()))
    }
}

#[cfg(test)]
mod tests {
    use crate::data_server_util::SaveFileInfo;
    use crate::game_config_util::ServerSettings;

    #[tokio::test]
    async fn test_get_game_config_by_serverconfig_id() {
        let url = "http://localhost:3000/api/game_master/game_config?serverconfig_id=1";
        let response = reqwest::get(url).await.unwrap();

        if response.status().is_success() {
            let dog_api_response = response.json::<ServerSettings>().await.unwrap();
            println!("{:?}", dog_api_response);
        } else {
            println!("请求失败，状态码: {}", response.status());
        }
    }

    #[tokio::test]
    async fn test_get_savefile_info_by_save_file_id() {
        let url = "http://192.168.8.88:3000/api/game_master/download_savefile?save_file_id=1";
        let response = reqwest::get(url).await.unwrap();

        if response.status().is_success() {
            let dog_api_response = response.json::<SaveFileInfo>().await.unwrap();
            println!("{:?}", dog_api_response);
        } else {
            println!("请求失败，状态码: {}", response.status());
        }
    }
}