/// Gets the file name stem of the key
fn key_type_to_stem(key_type: &str) -> Result<String, &'static str> {
    return match key_type {
        "ssh-ed25519" => Ok("id_ed25519".into()),
        "ssh-rsa" => Ok("id_rsa".into()),
        "ecdsa-sha2-nistp256" |
            "ecdsa-sha2-nistp384" |
            "ecdsa-sha2-nistp521" => Ok("id_ecdsa".into()),
        "ssh-dss" => Ok("id_dsa".into()),
        "sk-ssh-ed25519@openssh.com" => Ok("id_ed25519_sk".into()),
        "sk-ecdsa-sha2-nistp256@openssh.com" |
            "sk-ecdsa-sha2-nistp384@openssh.com" |
            "sk-ecdsa-sha2-nistp521@openssh.com" => Ok("id_ecdsa_sk".into()),
        _ => Err("Invalid key type"),
    }
}

fn main() {
    println!("Hello, world!")
}
