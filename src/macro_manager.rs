use crate::event::MacroEvent;

use anyhow::Result;
use log::debug;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMacro {
    pub name: String,
    pub events: Vec<MacroEvent>,
    pub created_at: u64,
    // pub updated_at: u64,
}

#[derive(Debug)]
pub struct MacroManager {
    macros: Arc<Mutex<HashMap<String, SavedMacro>>>,
    storage_path: String,
}

impl MacroManager {
    pub fn new() -> Self {
        let storage_path = "macros".to_string();

        // 确保存储目录存在
        if !Path::new(&storage_path).exists() {
            if let Err(e) = fs::create_dir_all(&storage_path) {
                debug!("Failed to create macros directory: {e}");
            }
        }

        let manager = Self {
            macros: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };

        // 加载已保存的宏
        if let Err(e) = manager.load_all_macros() {
            debug!("Failed to load macros: {e}");
        }

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

        self.macros.lock().insert(name.to_string(), saved_macro);
        Ok(())
    }

    pub fn load_macro(&self, name: &str) -> Result<Option<Vec<MacroEvent>>> {
        let file_path = format!("{}/{}.json", self.storage_path, name);

        if !Path::new(&file_path).exists() {
            return Ok(None);
        }

        let content = fs::read(file_path)?;
        let saved_macro: SavedMacro = serde_json::from_slice(&content)?;

        // 更新内存中的宏
        self.macros.lock().insert(name.to_string(), saved_macro.clone());

        Ok(Some(saved_macro.events))
    }

    pub fn delete_macro(&self, name: &str) -> Result<()> {
        let file_path = format!("{}/{}.json", self.storage_path, name);

        if Path::new(&file_path).exists() {
            fs::remove_file(file_path)?;
        }

        self.macros.lock().remove(name);
        Ok(())
    }

    pub fn rename_macro(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = format!("{}/{}.json", self.storage_path, old_name);
        let new_path = format!("{}/{}.json", self.storage_path, new_name);

        if Path::new(&old_path).exists() {
            fs::rename(old_path, &new_path)?;
            let macro_data = self.macros.lock().remove(old_name);
            if let Some(mut macro_data) = macro_data {
                macro_data.name = new_name.to_string();
                fs::write(new_path, serde_json::to_string(&macro_data)?)?;
                self.macros.lock().insert(new_name.to_string(), macro_data);
            }
        }

        Ok(())
    }

    pub fn get_all_macros(&self) -> Vec<SavedMacro> {
        self.macros.lock().values().cloned().collect()
    }

    pub fn get_macro_names(&self) -> Vec<String> {
        self.macros.lock().keys().cloned().collect()
    }

    pub fn macro_exists(&self, name: &str) -> bool {
        self.macros.lock().contains_key(name)
    }

    fn load_all_macros(&self) -> Result<()> {
        let dir = fs::read_dir(&self.storage_path)?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(content) = fs::read(&path) {
                        if let Ok(saved_macro) = serde_json::from_slice::<SavedMacro>(&content) {
                            self.macros.lock().insert(name.to_string(), saved_macro);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_macros(&self, names: &[String]) -> Vec<SavedMacro> {
        self.macros
            .lock()
            .values()
            .filter(|m| names.contains(&m.name))
            .cloned()
            .collect()
    }
}
