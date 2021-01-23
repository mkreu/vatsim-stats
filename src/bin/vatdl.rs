use log::error;
use std::{env, error::Error};
use vatsim_stats::vatdl::{run_compress, run_download};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    if let Some(arg) = env::args().nth(1) {
        if &arg == "compress" {
            if let Err(e) = run_compress().await {
                error!("Vatsim datafeed compression failed with error: {}", e);
                Err(e)?;
            }
        } else if &arg == "download" {
            if let Err(e) = run_download().await {
                error!("Vatsim download failed with unrecoverable error:{}", e);
                Err(e)?;
            }
        } else {
            Err("invalid argument")?;
        }
    }
    Ok(())
}
