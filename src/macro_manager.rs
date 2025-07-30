use crate::event::MacroEvent;

use anyhow::Result;
use autopilot::alert;
use log::debug;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::BufReader, path::Path, sync::Arc, thread};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMacro {
    pub name: String,
    pub events: Vec<MacroEvent>,
    pub created_at: u64,
    // pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct MacroManager {
    pub macros: Arc<RwLock<BTreeMap<String, SavedMacro>>>,
    storage_path: String,
}

impl MacroManager {
    pub fn new() -> Self {
        // 使用用户主目录下的应用程序数据目录
        let storage_path = if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".mousepilot").join("macros").to_string_lossy().to_string()
        } else {
            // 回退到当前目录
            "macros".to_string()
        };
        debug!("storage_path: {storage_path}");

        // alert::alert(&storage_path, Some("alert"), None, None);

        // 确保存储目录存在
        if !Path::new(&storage_path).exists() {
            if let Err(e) = fs::create_dir_all(&storage_path) {
                debug!("Failed to create macros directory: {e}");
                alert::alert(&e.to_string(), Some("alert"), None, None);
            }
        }

        let manager = Self {
            macros: Default::default(),
            storage_path,
        };

        let manager_clone = manager.clone();
        thread::spawn(move || {
            // 加载已保存的宏
            if let Err(e) = manager_clone.load_all_macros() {
                debug!("Failed to load macros: {e}");
            }
        });

        manager
    }

    pub fn save_macro(&self, name: &str, events: Vec<MacroEvent>) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let saved_macro = SavedMacro {
            name: name.to_string(),
            events,
            created_at: now,
        };

        let file_path = format!("{}/{}.json", self.storage_path, name);
        let json = serde_json::to_string(&saved_macro)?;
        fs::write(file_path, json)?;

        self.macros.write().insert(name.to_string(), saved_macro);
        Ok(())
    }

    pub fn load_macro(&self, name: &str) -> Result<Option<Vec<MacroEvent>>> {
        let file_path = format!("{}/{}.json", self.storage_path, name);

        if !Path::new(&file_path).exists() {
            return Ok(None);
        }

        let file = fs::File::open(file_path)?;
        let saved_macro: SavedMacro = serde_json::from_reader(BufReader::new(file))?;

        // 更新内存中的宏
        self.macros.write().insert(name.to_string(), saved_macro.clone());

        Ok(Some(saved_macro.events))
    }

    pub fn delete_macro(&self, name: &str) -> Result<()> {
        let file_path = format!("{}/{}.json", self.storage_path, name);

        if Path::new(&file_path).exists() {
            fs::remove_file(file_path)?;
        }

        self.macros.write().remove(name);
        Ok(())
    }

    pub fn rename_macro(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = format!("{}/{}.json", self.storage_path, old_name);
        let new_path = format!("{}/{}.json", self.storage_path, new_name);

        if Path::new(&old_path).exists() {
            fs::rename(old_path, &new_path)?;
            let macro_data = self.macros.write().remove(old_name);
            if let Some(mut macro_data) = macro_data {
                macro_data.name = new_name.to_string();
                fs::write(new_path, serde_json::to_string(&macro_data)?)?;
                self.macros.write().insert(new_name.to_string(), macro_data);
            }
        }

        Ok(())
    }

    pub fn get_all_macros(&self) -> Vec<SavedMacro> {
        self.macros.read().values().cloned().collect()
    }

    pub fn get_macro_count(&self) -> usize {
        self.macros.read().len()
    }

    pub fn get_macro_names(&self) -> Vec<String> {
        self.macros.read().keys().cloned().collect()
    }

    pub fn macro_exists(&self, name: &str) -> bool {
        self.macros.read().contains_key(name)
    }

    fn load_all_macros(&self) -> Result<()> {
        let dir = fs::read_dir(&self.storage_path)?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(file) = fs::File::open(&path) {
                        if let Ok(saved_macro) = serde_json::from_reader(BufReader::new(file)) {
                            self.macros.write().insert(name.to_string(), saved_macro);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_macros(&self, names: &[String]) -> Vec<SavedMacro> {
        self.macros
            .read()
            .values()
            .filter(|m| names.contains(&m.name))
            .cloned()
            .collect()
    }
}
