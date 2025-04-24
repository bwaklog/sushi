pub mod device;

use std::sync::Arc;

use clap::Parser;
use device::VPN;
use tokio::{io::AsyncWriteExt, signal};
use tracing::{error, info, Level};
use tracing_subscriber::fmt::time;

#[derive(Parser)]
struct Args {
    /// Flag for starting sushi as a VPN server
    #[arg(short)]
    server: bool,

    /// TUN interface address for VPN
    #[arg(long, short)]
    tip: Option<String>,

    /// Local address of interface used to talk with the VPN server
    #[arg(long, short)]
    local: String,

    /// Remote address of the VPN server
    #[arg(long, short)]
    remote: Option<String>,
}

#[tokio::main]
async fn main() {
    print!(
        "
    ███████╗██╗   ██╗███████╗██╗  ██╗██╗
    ██╔════╝██║   ██║██╔════╝██║  ██║██║
    ███████╗██║   ██║███████╗███████║██║
    ╚════██║██║   ██║╚════██║██╔══██║██║
    ███████║╚██████╔╝███████║██║  ██║██║
    ╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝
    "
    );

    let _ = tokio::io::stdout().flush().await;

    info!("TUN Interfaces!");
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_timer(time::ChronoLocal::rfc_3339())
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let vnic = VPN::init(device::Config {
        server: args.server,
        tun_ip: args.tip,
        udp_local_ip: args.local,
        udp_remote_ip: args.remote,
    })
    .await;

    let vnic_clone = Arc::new(vnic);
    let vnic_device_clone = vnic_clone.clone();

    if args.server {
        tokio::spawn(async move {
            vnic_device_clone.vpn_forwarder().await;
        });
    } else {
        tokio::spawn(async move {
            vnic_clone.tun_listener().await;
        });

        tokio::spawn(async move {
            vnic_device_clone.udp_listener().await;
        });
    }

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            error!("Unable to listen to shutdown singal {:?}", err);
        }
    }
}
