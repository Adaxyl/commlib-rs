use commlib_sys::data_schema::DataSchema;
use std::any::Any;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};
#[derive(Eq, Hash, PartialEq, std::fmt::Debug)]
pub enum ConfigCid {
    Cid_Role = 1,
    Cid_Game = 2,
}

pub trait ConfigTable: Send + Sync + Any {
    fn get_cid(&self) -> ConfigCid;
    //关注的table
    fn get_cared_table(&self) -> Vec<String>;
    //加载配置
    fn load(&mut self, ds: Box<DataSchema>) -> bool;
    fn clear(&mut self);
}

#[derive(Clone)]
pub struct WrappedConfigTable(pub Arc<Mutex<Box<dyn ConfigTable + Send>>>);

impl PartialEq for WrappedConfigTable {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for WrappedConfigTable {}
impl Hash for WrappedConfigTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0);
        ptr.hash(state);
    }
}
