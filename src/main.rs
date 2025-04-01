pub mod device;

use std::sync::Arc;

use clap::Parser;
use device::VNIC;
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::time;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    tip: String,

    #[arg(long)]
    ulocal: String,

    #[arg(long)]
    uremote: String,
}

#[tokio::main]
async fn main() {
    info!("TUN Interfaces!");
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_timer(time::ChronoLocal::rfc_3339())
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let vnic = VNIC::init(device::Config {
        tun_ip: args.tip,
        udp_local_ip: args.ulocal,
        udp_remote_ip: args.uremote,
    })
    .await;

    let vnic_clone = Arc::new(vnic);
    let vnic_device_clone = vnic_clone.clone();

    tokio::spawn(async move {
        vnic_clone.tun_listener().await;
    });
    tokio::spawn(async move {
        vnic_device_clone.udp_listener().await;
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            error!("Unable to listen to shutdown singal {:?}", err);
        }
    }
}
