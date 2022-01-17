use serde_yaml::Value;
use std::{fs::File, io::Read, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Inlines references in kubeconfig into the config")]
struct Opt {
    /// File name: only required when `out-type` is set to `file`
    #[structopt(name = "FILE")]
    file_name: PathBuf,
}

fn replace_refs(config: &mut Value, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(obj) = config.as_mapping_mut() {
        let key_value = Value::String(key.to_string());
        if let Some(Value::String(s)) = obj.get(&key_value) {
            if let Ok(ref mut file) = File::open(s) {
                let mut contents = vec![];
                file.read_to_end(&mut contents)?;
                obj.remove(&key_value);
                let new_key = Value::String(format!("{}-data", key));
                obj.insert(new_key, Value::String(base64::encode(contents)));
            }
        }
    }
    Ok(())
}

fn replace_cluster(clusters: &mut Value) -> Result<(), Box<dyn std::error::Error>> {
    for cluster in clusters.as_sequence_mut().unwrap_or(&mut Vec::new()) {
        if let Some(cluster) = cluster.get_mut("cluster") {
            replace_refs(cluster, "certificate-authority")?;
        }
    }
    Ok(())
}

fn replace_users(users: &mut Value) -> Result<(), Box<dyn std::error::Error>> {
    for user in users.as_sequence_mut().unwrap_or(&mut Vec::new()) {
        if let Some(user) = user.get_mut("user") {
            replace_refs(user, "client-certificate")?;
            replace_refs(user, "client-key")?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let file = File::open(opt.file_name)?;
    let mut config: Value = serde_yaml::from_reader(file)?;
    replace_cluster(&mut config["clusters"])?;
    replace_users(&mut config["users"])?;
    serde_yaml::to_writer(std::io::stdout(), &config)?;
    Ok(())
}
