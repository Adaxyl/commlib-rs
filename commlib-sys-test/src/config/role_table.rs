use commlib_sys::data_schema::DataSchema;
use hashbrown::HashMap;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::sync::{Arc, Mutex};

use crate::config_table::ConfigCid;
use crate::config_table::ConfigTable;
lazy_static::lazy_static! {
     pub static ref ROLE_CONFIG: Arc<Mutex<RoleTable>> = Arc::new(Mutex::new(RoleTable::new()));
}
impl ConfigTable for RoleTable {
    fn get_cid(&self) -> ConfigCid {
        ConfigCid::Cid_Role
    }
    //关注的table
    fn get_cared_table(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        v.push("roletable".to_string());
        return v;
    }

    //加载配置
    fn load(&mut self, ds: Box<DataSchema>) -> bool {
        if let Some(table) = ds.get_table("roletable") {
            for (i, v) in table.rows.iter().enumerate() {
                let mut conf = RoleConfig::new();
                if let Some(id) = table.get_value::<u32>(i, "id") {
                    conf.id = id;
                    log::info!("id:{:?}", id);
                }

                if let Some(name) = table.get_value::<String>(i, "name") {
                    conf.name = name;
                }
                self.datas.insert(conf.id, conf);
            }
            return true;
        }
        return false;
    }
    fn clear(&mut self) {}
}
///
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RoleConfig {
    pub id: u32,
    pub name: String,
    pub config_id: u32,
}
impl RoleConfig {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            config_id: 0,
        }
    }
}

pub struct RoleTable {
    pub datas: HashMap<u32, RoleConfig>,
}
impl RoleTable {
    pub fn new() -> Self {
        Self {
            datas: HashMap::new(),
        }
    }
    pub fn get_instance() -> Arc<Mutex<RoleTable>> {
        ROLE_CONFIG.clone()
    }
    pub fn get_role_config(&self, id: u32) -> Option<&RoleConfig> {
        self.datas.get(&id)
    }
    pub fn get_role_configs(&self) -> &HashMap<u32, RoleConfig> {
        &self.datas
    }
}
