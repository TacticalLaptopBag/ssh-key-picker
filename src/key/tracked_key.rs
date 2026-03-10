use serde::{Deserialize, Serialize};

use crate::key::KeyType;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackedKey {
    pub name: String,
    pub key_type: KeyType,
}
