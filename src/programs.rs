use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Programs {
    path: Vec<PathBuf>,
    cache: HashMap<String, PathBuf>,
}

impl Default for Programs {
    fn default() -> Self {
        Self {
            path: std::env::vars()
                .collect::<HashMap<String, String>>()
                .get("PATH")
                .unwrap()
                .split([':', ' '])
                .map(PathBuf::from)
                .collect(),
            cache: HashMap::new(),
        }
    }
}

impl Programs {
    pub fn find(&mut self, program: impl Into<String>) -> Option<PathBuf> {
        let program = program.into();
        if let Some(program) = self.cache.get(&program) {
            return Some(program.clone());
        }
        for bin in self.path.iter() {
            let full_path = bin.join(&program);
            if full_path.is_file() {
                self.cache.insert(program, full_path.clone());
                return Some(full_path);
            }
        }

        None
    }
}
