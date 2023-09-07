use commlib_sys::data_schema::DataSchema;
use commlib_sys::ServiceRs;

use crate::config_table::ConfigCid;
use crate::config_table::ConfigTable;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::sync::Arc;
use std::sync::Mutex;
lazy_static::lazy_static! {
     pub static ref G_CONFIG: Arc<Mutex<GConfigTable>> = Arc::new(Mutex::new(GConfigTable::new()));
}
impl ConfigTable for GConfigTable {
    fn get_cid(&self) -> ConfigCid {
        ConfigCid::Cid_Game
    }
    //关注的table
    fn get_cared_table(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        v.push("gconfig".to_string());
        return v;
    }
    //加载配置
    fn load(&mut self, ds: Box<DataSchema>) -> bool {
        if let Some(table) = ds.get_table("gconfig") {
            for (i, v) in table.rows.iter().enumerate() {
                //   if let Some(name) = table.get_value::<String>(i, "NormalMissionCount") {
                //      log::info!("name:{:?}", name);
                //   }
            }
            return true;
        }
        return false;
    }
    fn clear(&mut self) {}
}
///
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct GConfigTable {
    pub id: u32,
    pub a1: u32,
    pub s1: String,
}
impl GConfigTable {
    pub fn new() -> Self {
        Self {
            id: 0,
            a1: 0,
            s1: String::new(),
        }
    }
    pub fn get_instance() -> Arc<Mutex<GConfigTable>> {
        G_CONFIG.clone()
    }
}
