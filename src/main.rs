use anyhow::Result;
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
    let list = cmd.list_parallel(&svn_path)?;
    println!(
        "time took with SVN: {:#?} msec",
        Instant::now().duration_since(instant).as_millis()
    );
    list.lock()
        .unwrap()
        .iter()
        .for_each(|e| println!("{:?}", e));
    println!(
        "time took: {:#?} msec",
        Instant::now().duration_since(instant).as_millis()
    );
    Ok(())
}
