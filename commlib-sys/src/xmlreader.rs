use crate::DataTable;

#[derive(Default, Debug, Clone)]
pub struct XmlReader {
    pub key: String,
    pub value: String,
    children: hashbrown::HashMap<String, Vec<XmlReader>>,
}

static XML_READER_EMPTY_LIST: Vec<XmlReader> = Vec::<XmlReader>::new();

/// 如果 Vec 存在则直接插入，如果 Vec 不存在则新建并插入
fn insert_child_reader(node_reader: &mut XmlReader, child_reader: XmlReader) {
    let checkopt = node_reader.children.get_mut(&child_reader.key);
    match checkopt {
        Some(v) => v.push(child_reader),
        None => {
            let key = child_reader.key.clone();
            let mut new_vec = Vec::<XmlReader>::with_capacity(16);
            new_vec.push(child_reader);
            node_reader.children.insert(key, new_vec);
        }
    }
}

impl XmlReader {
    ///
    fn do_parse(node: &roxmltree::Node) -> Self {
        let mut node_reader = Self::new();
        node_reader.key = node.tag_name().name().to_string();
        if let Some(val) = node.text() {
            node_reader.value = val.to_string();
        }

        // walk attributes
        for i in 0..node.attributes().len() {
            let attr = node.attributes().nth(i).unwrap();
            let mut attr_reader = Self::new();

            attr_reader.key = attr.name().to_string();
            attr_reader.value = attr.value().to_string();

            insert_child_reader(&mut node_reader, attr_reader);
        }

        // walk children nodes
        for child_node in node.children() {
            if child_node.is_element() {
                let child_reader = Self::do_parse(&child_node);
                insert_child_reader(&mut node_reader, child_reader);
            }
        }

        //
        node_reader
    }

    /// Constructor
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    // 从文件构造 XmlReader 对象
    pub fn read_file(path: &std::path::Path) -> Result<Self, String> {
        // 读取文件到内存并解析
        let content_r = std::fs::read_to_string(path);
        match content_r {
            Ok(content) => Self::read_content(&content),
            Err(e) => {
                let errmsg = format!("parse xml file({:?}) error: {}.", path, e);
                println!("{errmsg}");
                Err(errmsg)
            }
        }
    }

    // 从字符串构造 XmlReader 对象
    pub fn read_content(content: &str) -> Result<Self, String> {
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let doc = match roxmltree::Document::parse_with_options(&content, opt) {
            Ok(doc) => doc,
            Err(e) => {
                let errmsg = format!(
                    "parse xml content failed!!! error: {}, len: {}.",
                    e,
                    content.len()
                );
                println!("{errmsg}");
                return Err(errmsg);
            }
        };

        // XmlReader
        let root_reader = Self::do_parse(&doc.root_element());
        Ok(root_reader)
    }

    /// 根据 键值路径(keys) 读取 节点 字符串值
    pub fn get_string(&self, keys: Vec<&str>, default_value: &str) -> String {
        if let Some(reader) = self.get_child(keys) {
            reader.value.clone()
        } else {
            default_value.to_owned()
        }
    }

    /// 根据 键值路径(keys) 读取 节点 u64值
    pub fn get_u64(&self, keys: Vec<&str>, default_value: u64) -> u64 {
        if let Some(reader) = self.get_child(keys) {
            if let Ok(n) = reader.value.parse::<u64>() {
                n
            } else {
                0
            }
        } else {
            default_value
        }
    }

    /// 根据 键值路径(keys) 读取 节点 字符串值，然后转换成目标类型 T
    pub fn get<T>(&self, keys: Vec<&str>, default_value: T) -> T
    where
        T: std::str::FromStr + ToOwned<Owned = T>,
    {
        if let Some(reader) = self.get_child(keys) {
            if let Ok(v) = reader.value.parse::<T>() {
                v
            } else {
                default_value
            }
        } else {
            default_value
        }
    }

    /// 根据 键值路径(keys) 查找 节点, 遇到多 children 直接选取第一个child
    pub fn get_child(&self, keys: Vec<&str>) -> Option<&Self> {
        if 0 == keys.len() {
            return None;
        }

        let mut cur = self;

        for key in keys {
            let checkopt = cur.children.get(key);
            match checkopt {
                Some(v) => {
                    if let Some(next) = v.first() {
                        cur = next;
                    } else {
                        return None;
                    }
                }
                None => {
                    return None;
                }
            }
        }
        Some(cur)
    }

    /// 根据 键值路径(keys) 读取 节点 列表
    pub fn get_children(&self, keys: Vec<&str>) -> Option<&Vec<Self>> {
        if 0 == keys.len() {
            return None;
        }

        let mut cur = self;
        let mut list = &XML_READER_EMPTY_LIST;

        for key in keys {
            let checkopt = cur.children.get(key);
            match checkopt {
                Some(v) => {
                    list = v;

                    //
                    if let Some(next) = v.first() {
                        cur = next;
                    }
                }
                None => {
                    return None;
                }
            }
        }
        Some(list)
    }
    //读取xml配置表
    pub fn read_data_table(path: &String) -> Result<DataTable, String> {
        // 读取文件到内存并解析
        let content_r = std::fs::read_to_string(path);
        match content_r {
            Ok(content) => Self::read_data_string(&content),
            Err(e) => {
                let errmsg = format!("parse xml file({:?}) error: {}.", path, e);
                println!("{errmsg}");
                Err(errmsg)
            }
        }
    }
    pub fn get_data_table_fields(node: &roxmltree::Node) -> Vec<String> {
        let mut attrs: Vec<String> = Vec::new();
        // 遍历 data 节点
        for data_node in node
            .children()
            .filter(|node| node.tag_name().name() == "data")
        {
            // 遍历 cell 节点
            for cell_node in data_node
                .descendants()
                .filter(|node| node.tag_name().name() == "cell")
            {
                let name = cell_node
                    .attribute("name")
                    .expect("Attribute 'name' not found");
                let content = cell_node.text().expect("Cell content not found");
                if attrs.contains(&name.to_string()) {
                    continue;
                }
                attrs.push((&name).to_string());
                log::debug!("Name: {}, Content: {}", name, content);
            }
        }
        attrs
    }
    pub fn read_data_string(xml_data: &str) -> Result<DataTable, String> {
        if xml_data.len() == 0 {
            let errmsg = format!("xml_data.len() == 0 ");
            return Err(errmsg);
        }
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let doc = match roxmltree::Document::parse_with_options(&xml_data, opt) {
            Ok(doc) => doc,
            Err(e) => {
                let errmsg = format!(
                    "parse xml xml_data failed!!! error: {}, len: {}.",
                    e,
                    xml_data.len()
                );
                println!("{errmsg}");
                return Err(errmsg);
            }
        };
        //doc
        let mut dt = DataTable::new(
            doc.root_element().tag_name().name().to_string(),
            Self::get_data_table_fields(&doc.root_element()),
        );

        if dt.fields.is_empty() {
            return Err("dt.fields.is_empty()".to_string());
        }
        let mut row_datas: Vec<Vec<String>> = Vec::new();
        // 遍历 cell 节点
        for data_node in doc
            .root_element()
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "data")
        {
            let mut rowstrs: Vec<String> = Vec::new();
            for cell_node in data_node
                .children()
                .filter(|node| node.is_element() && node.tag_name().name() == "cell")
            {
                let content = cell_node.text().expect("Cell content not found");

                println!(" Content: {}", content);
                rowstrs.push(content.to_string());
            }
            row_datas.push(rowstrs);
        }

        dt.set_data(row_datas);
        Ok(dt)
    }
}
