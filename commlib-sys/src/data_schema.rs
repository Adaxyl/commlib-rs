use crate::{xmlreader, ServiceRs, XmlReader};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::{collections::HashMap, str::FromStr};
use std::{fs, thread};

#[derive(Default, Debug, Clone)]
pub struct DataTable {
    pub name: String,
    pub fields: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub field_index: HashMap<String, usize>,
    pub rows_by_pk: HashMap<String, usize>,
}

impl DataTable {
    pub fn new(name: String, fields: Vec<String>) -> DataTable {
        DataTable {
            name,
            fields,
            rows: Vec::new(),
            field_index: HashMap::new(),
            rows_by_pk: HashMap::new(),
        }
    }
    pub fn set_data(&mut self, data: Vec<Vec<String>>) {
        // 使用第一个字段作为参考字段
        let reference_field = &self.fields[0];

        // 将传入的数据复制到rows字段
        self.rows = data.clone();

        // 找到参考字段在fields中的索引
        if let Some(reference_field_index) = self
            .fields
            .iter()
            .position(|field| field == reference_field)
        {
            // 清空字段索引和主键索引
            self.field_index.clear();
            self.rows_by_pk.clear();

            // 更新字段索引
            for (index, field) in self.fields.iter().enumerate() {
                self.field_index.insert(field.clone(), index);
            }

            // 遍历每一行数据
            for (row_index, row) in self.rows.iter().enumerate() {
                // 获取当前行的主键值（使用参考字段的值作为主键）
                let key = row.get(reference_field_index).cloned().unwrap_or_default();
                // 将主键值和行索引插入主键索引中
                self.rows_by_pk.insert(key, row_index);
            }
        }
    }
    pub fn get(&self, row: usize, column: &str) -> String {
        let mut index_row = 0;

        if row < self.rows.len() && row > 0 {
            index_row = row;
        }
        let data = &self.rows[index_row];
        let it = self.field_index.get(column).copied().unwrap_or(0);

        if data.len() < self.field_index.len() {
            panic!("data.len < self.field_index.len");
        }
        if it < data.len() {
            if let Some(tmp) = data.get(it) {
                return tmp.clone();
            } else {
                return String::new();
            }
        } else {
            panic!(" it < data.len()");
        }
    }
    //获取不同类型字段
    pub fn get_value<T>(&self, row: usize, column: &str) -> Option<T>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        let data = self.get(row, column);
        if !data.is_empty() {
            return data.parse().ok();
        }
        None
    }

    fn get_row_by_key(&self, key: &str) -> Option<usize> {
        self.rows_by_pk.get(key).copied()
    }
}
#[derive(Debug, Clone)]
pub struct DataSchema {
    pub tables: HashMap<String, DataTable>,
}

impl DataSchema {
    pub fn new() -> Self {
        DataSchema {
            tables: HashMap::new(),
        }
    }

    pub fn get_table(&self, name: &str) -> Option<&DataTable> {
        self.tables.get(name)
    }

    pub fn to_simple_info(&self) -> String {
        String::new()
    }
}

struct DataSchemaLoader {
    myid: i32,
    dc: Arc<Mutex<DataSchema>>, //多线程读写
    cb: Box<dyn FnMut(Box<DataSchema>) + Send + Sync>,
    need_load_tables: Vec<String>,
    xml_path: String,
    xml_data: String,
    count: u32,
    ec: i32,
    tables: HashMap<String, bool>,
    pks: HashMap<String, String>,
}

impl DataSchemaLoader {
    pub fn new() -> Self {
        fn default_closure() -> Box<dyn FnMut(Box<DataSchema>) + Send + Sync> {
            Box::new(|_| {})
        }
        DataSchemaLoader {
            myid: 0,
            dc: Arc::new(Mutex::new(DataSchema::new())),
            need_load_tables: Vec::new(),
            count: 0,
            ec: 0,
            tables: HashMap::new(),
            pks: HashMap::new(),
            cb: default_closure(),
            xml_path: String::new(),
            xml_data: String::new(),
        }
    }

    pub fn do_load_xml<T>(&mut self, srv: &Arc<T>)
    where
        T: ServiceRs + 'static,
    {
        for v in &self.need_load_tables {
            let file_path = format!("{}{}", self.xml_path, v);

            let dt = XmlReader::read_data_table(&file_path);
            match dt {
                Ok(content) => {
                    let key = &content.name;
                    let value = &content.fields[0];
                    self.pks.insert(key.to_string(), value.to_string());
                    // 写数据锁，可以同时被多个线程获取
                    let mut write_lock = self.dc.lock().unwrap();
                    write_lock.tables.insert(key.to_string(), content.clone());
                    self.tables.insert(key.to_string(), true);
                }
                Err(_err) => {
                    continue;
                }
            }
        }
        let data_schema = self.dc.lock().unwrap();
        let db = Box::new(data_schema.clone());
        let cbb = self.cb.as_mut();
        let f = (*cbb)(db);
        srv.run_in_service(Box::new(move || f));
    }

    pub fn load_xml<T>(&mut self, srv: &Arc<T>)
    where
        T: ServiceRs + 'static,
    {
        let thread_handle = thread::scope(|s| {
            self.do_load_xml(srv);
        });
    }
}

fn get_just_current_file(cate_dir: &str) -> Vec<String> {
    let path = std::env::current_dir().unwrap().join(cate_dir);

    let mut file_list = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            file_list.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }

    file_list
}
pub fn load_data_schema_from_xml<T>(
    srv: &Arc<T>,
    path: &str,
    cb: Box<dyn FnMut(Box<DataSchema>) + Send + Sync>,
) where
    T: ServiceRs + 'static,
{
    let mut loader = DataSchemaLoader::new();
    loader.cb = cb;
    loader.dc = Arc::new(Mutex::new(DataSchema::new()));
    loader.xml_path = path.to_string();
    let allfiles = get_just_current_file(path);
    for i in allfiles {
        loader.need_load_tables.push(i.clone());
    }
    loader.load_xml(srv);
}
