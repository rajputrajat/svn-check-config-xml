use anyhow::Result;
use async_std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use std::{env, time::Instant};
use svn_cmd::{Credentials, SvnCmd};
use svn_list_parallel_rs::ListParallel;
use uuid::Uuid;

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
    let list = cmd.list_parallel(&svn_path)?;
    println!(
        "time took with SVN: {:#?} msec",
        Instant::now().duration_since(instant).as_millis()
    );
    let mut config_handler = ConfigFiles::new().await?;
    for e in list.lock().unwrap().iter() {
        let path = format!("{}/{}", e.0 .0, e.1.name);
        if path.contains("configuration.xml") {
            println!("{:?}", e);
            async {
                let file_text = cmd.cat(&path).await.unwrap();
                config_handler
                    .save_new_file(&path, &file_text)
                    .await
                    .unwrap();
            }
            .await;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct DbConfig {
    pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize)]
struct Pair {
    file_name: String,
    config_svn_url: String,
}

impl ConfigFiles {
    async fn new() -> Result<Self> {
        let dir_path = Path::new(&env::var("USERPROFILE")?).join("config_xmls");
        let db = ConfigFiles::get_db(dir_path.to_str().unwrap()).await?;
        Ok(Self { dir: dir_path, db })
    }

    async fn get_db(path: &str) -> Result<DbConfig> {
        match fs::read_to_string(&path).await {
            Ok(t) => {
                let db = toml::from_str::<DbConfig>(&t)?;
                Ok(db)
            }
            Err(_) => {
                File::create(&path).await?;
                Ok(DbConfig { pairs: Vec::new() })
            }
        }
    }

    async fn set_db(path: &str, db: DbConfig) -> Result<()> {
        let text = toml::to_string::<DbConfig>(&db)?;
        fs::write(path, &text).await?;
        Ok(())
    }

    async fn save_new_file(&mut self, svn_url: &str, file_content: &str) -> Result<()> {
        let file_name = Uuid::new_v4().to_string();
        let file_path = PathBuf::new().join(&self.dir).join(&file_name);
        fs::write(&file_path, file_content).await?;
        self.db.pairs.push(Pair {
            file_name: file_name,
            config_svn_url: svn_url.to_owned(),
        });
        Ok(())
    }
}

struct ConfigFiles {
    dir: PathBuf,
    db: DbConfig,
}
