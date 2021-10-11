use anyhow::Result;
use async_std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use std::{env, time::Instant};
use svn_cmd::{Credentials, SvnCmd};
use svn_list_parallel_rs::ListParallel;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let instant = Instant::now();
    let svn_path = env::args().nth(1).expect("svn path is not given");
    let cmd = SvnCmd::new(
        Credentials {
            username: "svc-p-blsrobo".to_owned(),
            password: "Comewel@12345".to_owned(),
        },
        None,
    )?;
    let dir = Path::new(&env::var("USERPROFILE")?).join("config_xmls");
    let db = dir.join("map.toml");
    let list = cmd.list_parallel(&svn_path)?;
    println!(
        "time took with SVN: {:#?} msec",
        Instant::now().duration_since(instant).as_millis()
    );
    list.lock()
        .unwrap()
        .iter()
        .filter(|e| {
            let path = format!("{}/{}", e.0 .0, e.1.name);
            path.contains("configuration.xml")
        })
        .for_each(|e| println!("{:?}", e));
    println!(
        "time took: {:#?} msec",
        Instant::now().duration_since(instant).as_millis()
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct DbConfig {
    pairs: Vec<Pairs>,
}

#[derive(Serialize, Deserialize)]
struct Pairs {
    name: String,
    file_path: String,
}

async fn get_db(path: &str) -> Result<DbConfig> {
    match fs::read_to_string(path).await {
        Ok(t) => {
            let db = toml::from_str::<DbConfig>(&t)?;
            Ok(db)
        }
        Err(_) => {
            File::create(path).await?;
            Ok(DbConfig { pairs: Vec::new() })
        }
    }
}

async fn set_db(path: &str, db: DbConfig) -> Result<()> {
    let text = toml::to_string::<DbConfig>(&db)?;
    fs::write(path, &text).await?;
    Ok(())
}
