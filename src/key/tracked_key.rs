use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::key::KeyType;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackedKey {
    pub name: String,
    pub key_type: KeyType,
}

#[derive(Debug, Clone)]
pub struct KeyPaths {
    pub private: PathBuf,
    pub public: PathBuf,
}

impl TrackedKey {
    pub fn get_enabled_paths(&self, keys_dir: &PathBuf) -> KeyPaths {
        let private_name = self.key_type.to_file_name();
        let private_path = keys_dir.join(private_name);
        let public_path = private_path.with_extension("pub");

        KeyPaths { 
            private: private_path,
            public: public_path,
        }
    }

    pub fn get_disabled_paths(&self, disabled_dir: &PathBuf) -> KeyPaths {
        let disabled_path = disabled_dir.join(&self.name);
        let disabled_pub_path = disabled_path.with_extension("pub");

        KeyPaths {
            private: disabled_path,
            public: disabled_pub_path,
        }
    }
}
