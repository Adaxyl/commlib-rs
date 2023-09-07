use commlib_sys::data_schema::DataSchema;
use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use crate::{
    config::{GConfigTable, RoleTable},
    config_table::{ConfigCid, ConfigTable},
};
lazy_static::lazy_static! {
     pub static ref CONFIG_MANAGER: Arc<Mutex<ConfigManager>> = Arc::new(Mutex::new(ConfigManager::new()));
}
pub struct ConfigManager {
    config_tables: HashMap<ConfigCid, Arc<Mutex<dyn ConfigTable>>>,
}
unsafe impl Sync for ConfigManager {}
impl ConfigManager {
    pub fn new() -> Self {
        ConfigManager {
            config_tables: HashMap::new(),
        }
    }
    pub fn get_instance() -> Arc<Mutex<ConfigManager>> {
        CONFIG_MANAGER.clone()
    }
    pub fn register(&mut self, config: Arc<Mutex<dyn ConfigTable>>) {
        let mut ac = config.lock().unwrap();
        self.config_tables.insert(ac.get_cid(), config.clone());
    }

    pub fn init(&mut self) {
        self.register(GConfigTable::get_instance());
        self.register(RoleTable::get_instance());
    }

    pub fn reload_tables(&mut self, ds: Box<DataSchema>, tables: HashSet<String>) {
        let mut reloads: HashSet<Box<dyn ConfigTable>> = HashSet::new();
        for config in tables {
            //   if let Some(it) = self.config_tables.get(&config) {
            //todo
            //  reloads.insert(Box::new(it));
            //   }
        }
        for mut t in reloads {
            t.clear();
            t.load(ds.clone());
        }
    }
    pub fn reload_all(&mut self, ds: Box<DataSchema>) -> bool {
        for (_, config) in &mut self.config_tables {
            let mut ac = config.lock().unwrap();
            ac.clear();
        }
        for (_, config) in &mut self.config_tables {
            let mut ac = config.lock().unwrap();
            if !ac.load(ds.clone()) {
                log::error!("[config.cid ={:?}] load err", ac.get_cid());
                //  return false;
            }
        }
        return true;
    }
}
