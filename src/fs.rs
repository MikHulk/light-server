use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub enum FsNode {
    File(Vec<u8>),
    Dir(HashMap<String, FsNode>),
}

impl FsNode {
    pub fn from_fs(path: &str) -> Result<FsNode, Box<dyn std::error::Error + Send + Sync>> {
        fn process_file(path: &str) -> Result<FsNode, Box<dyn std::error::Error + Send + Sync>> {
            let content = fs::read(path)?;
            Ok(FsNode::File(content))
        }

        fn process_dir(path: &str) -> Result<FsNode, Box<dyn std::error::Error + Send + Sync>> {
            let mut dir_map = HashMap::new();
            let dir_content = fs::read_dir(path)?;
            for entry in dir_content.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        let file_type = entry.file_type()?;
                        if file_type.is_file() {
                            let node = process_file(path)?;
                            dir_map.insert(file_name, node);
                        } else if file_type.is_dir() {
                            let node = process_dir(path)?;
                            dir_map.insert(file_name, node);
                        }
                    }
                }
            }
            Ok(FsNode::Dir(dir_map))
        }

        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            Ok(process_dir(path)?)
        } else {
            Ok(process_file(path)?)
        }
    }

    pub fn get<'a>(&'a self, path: &[&'a str]) -> Option<&Vec<u8>> {
        match self {
            FsNode::Dir(content) => {
                match content.get(if path[0].is_empty() {
                    "index.html"
                } else {
                    path[0]
                }) {
                    Some(FsNode::File(content)) => {
                        if path.len() == 1 {
                            Some(content)
                        } else {
                            None
                        }
                    }
                    Some(node) => node.get(&path[1..]),
                    None => None,
                }
            }
            FsNode::File(content) => {
                if path.is_empty() {
                    Some(content)
                } else {
                    None
                }
            }
        }
    }
}
