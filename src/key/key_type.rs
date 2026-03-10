use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KeyType {
    Ed25519(bool),
    Rsa(bool),
    Dss(bool),
    Ecdsa256(bool),
    Ecdsa384(bool),
    Ecdsa521(bool),
}

fn full_key_type(stem: &str, sk: bool) -> String {
    if sk {
        format!("sk-{}@openssh.com", stem)
    } else {
        stem.into()
    }
}

fn full_file_name(stem: &str, sk: bool) -> String {
    if sk {
        format!("{}_sk", stem)
    } else {
        stem.into()
    }
}

impl KeyType {
    pub fn from_type(key_type: &str) -> Result<KeyType, String> {
        match key_type {
            "ssh-ed25519" => Ok(KeyType::Ed25519(false)),
            "ssh-rsa" => Ok(KeyType::Rsa(false)),
            "ssh-dss" => Ok(KeyType::Dss(false)),
            "ecdsa-ssh2-nistp256" => Ok(KeyType::Ecdsa256(false)),
            "ecdsa-ssh2-nistp384" => Ok(KeyType::Ecdsa384(false)),
            "ecdsa-ssh2-nistp521" => Ok(KeyType::Ecdsa521(false)),
            "sk-ssh-ed25519@openssh.com" => Ok(KeyType::Ed25519(true)),
            "sk-ssh-rsa@openssh.com" => Ok(KeyType::Rsa(true)),
            "sk-ssh-dss@openssh.com" => Ok(KeyType::Dss(true)),
            "sk-ecdsa-ssh2-nistp256@openssh.com" => Ok(KeyType::Ecdsa256(true)),
            "sk-ecdsa-ssh2-nistp384@openssh.com" => Ok(KeyType::Ecdsa384(true)),
            "sk-ecdsa-ssh2-nistp521@openssh.com" => Ok(KeyType::Ecdsa521(true)),
            _ => Err(format!("Unknown type {}", key_type)),
        }
    }

    pub fn to_string_type(&self) -> String {
        match self {
            KeyType::Ed25519(sk) => full_key_type("ssh-ed25519", *sk),
            KeyType::Rsa(sk) => full_key_type("ssh-rsa", *sk),
            KeyType::Dss(sk) => full_key_type("ssh-dss", *sk),
            KeyType::Ecdsa256(sk) => full_key_type("ecdsa-ssh2-nistp256", *sk),
            KeyType::Ecdsa384(sk) => full_key_type("ecdsa-ssh2-nistp384", *sk),
            KeyType::Ecdsa521(sk) => full_key_type("ecdsa-ssh2-nistp521", *sk),
        }.into()
    }

    pub fn to_file_name(&self) -> String {
        match self {
            KeyType::Ed25519(sk) => full_file_name("id_ed25519", *sk),
            KeyType::Rsa(sk) => full_file_name("id_rsa", *sk),
            KeyType::Dss(sk) => full_file_name("id_dsa", *sk),
            KeyType::Ecdsa256(sk) |
            KeyType::Ecdsa384(sk) |
            KeyType::Ecdsa521(sk) => full_file_name("id_ecdsa", *sk),
        }
    }
}
