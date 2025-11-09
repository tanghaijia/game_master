use anyhow::bail;
use serde::Serialize;
use tera::{Tera, Context};
use tokio::fs;
use crate::const_value::SERVERCONFIG_XML_PATH;

#[derive(Serialize)]
pub struct ServerSettings {
    pub server_name: String,
    pub server_description: String,
    pub server_password: String,
    pub language: String,
    pub server_max_player_count: i32,
    pub eac_enabled: bool,
    pub game_difficulty: i32,
    pub party_shared_kill_range: i32,
    pub player_killing_mode: i32
}

pub struct GameConfigUtil {
    init: bool,
    tera: Tera,
}

impl GameConfigUtil {
    pub fn new() -> Self {
        let mut tera = Tera::default();

        let rss_template = include_str!("../templates/serverconfig.xml");
        tera.add_raw_template("serverconfig.xml", rss_template).unwrap();

        GameConfigUtil { init: true, tera }
    }

    fn render(&self, server_settings: &ServerSettings) -> anyhow::Result<String> {
        let mut context = Context::new();
        context.insert("settings", server_settings);

        let xml = self.tera.render("serverconfig.xml", &context)?;
        println!("{}", xml);

        Ok(xml)
    }

    pub async fn set_serverconfig_xml(&self, server_settings: &ServerSettings) -> anyhow::Result<()> {
        if !self.init {
            bail!("GameConfigUtil not init");
        }

        let xml = self.render(server_settings)?;
        fs::write(SERVERCONFIG_XML_PATH, xml).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::game_config_util::GameConfigUtil;

    #[test]
    fn init_test() {

        GameConfigUtil::new();
    }
}