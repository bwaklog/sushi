use core::panic;
use std::{net::SocketAddr, process::exit};

use tokio::net::UdpSocket;
use tracing::{debug, error, warn};
use tun::{AbstractDevice, AsyncDevice};

pub struct Config {
    pub tun_ip: String,
    pub udp_local_ip: String,
    pub udp_remote_ip: String,
}

pub struct VNIC {
    pub device: AsyncDevice,
    udp: UdpSocket,
    config: Config,
}

pub const TUN_MTU: usize = 1504;

impl VNIC {
    pub async fn init(config: Config) -> VNIC {
        let mut configuration = tun::Configuration::default();
        configuration
            .address(config.tun_ip.clone())
            .netmask((255, 255, 255, 0))
            .up();

        #[cfg(target_os = "linux")]
        configuration.platform_config(|conf| {
            // requiring root privilege to acquire complete functions
            conf.ensure_root_privileges(true);
        });

        debug!(
            "Created TUN interface with address {:?}",
            config.tun_ip.clone()
        );

        let mut device: AsyncDevice;
        match tun::create_as_async(&configuration) {
            Ok(dev) => device = dev,
            Err(e) => {
                error!("failed to create an async device: {:?}", e);
                exit(1);
            }
        }

        // we encapsulate each packet in a UDP datagram and forward

        // with UDP headers, we have a 28 byte overhead
        // sending with the full TUN MTU size would cause fragmentation
        // so we set the TUN MTU to be lower
        if device.set_mtu(1472).is_err() {
            error!("failed to set TUN device {:?}", TUN_MTU);
            exit(1);
        }

        debug!(
            "created async TUN device, set MTU to {}",
            device.mtu().unwrap()
        );

        let udp: UdpSocket;
        debug!(
            "Trying to create UDP socket {:?}",
            config.udp_local_ip.clone().as_str()
        );
        let res = UdpSocket::bind(config.udp_local_ip.clone().as_str()).await;
        match res {
            Ok(socket) => udp = socket,
            Err(e) => {
                error!(
                    "Failed to create a udp socket {:?} => {:?}",
                    config.udp_local_ip.clone(),
                    e
                );
                panic!();
            }
        }
        debug!("Created a UDP socket {:?}", config.udp_local_ip.clone());

        VNIC {
            device,
            udp,
            config,
        }
    }

    pub async fn tun_listener(&self) {
        debug!("Listening on TUN interface...");
        let mut buf = vec![0; TUN_MTU];
        let config_remote_ip = self.config.udp_remote_ip.clone();

        loop {
            let size = self.device.recv(&mut buf).await.unwrap();
            if size > 4 {
                debug!(
                    "Recieved packet with size {:?} from TUN interface, forwarding to {:?}",
                    size, config_remote_ip
                );
            }

            match self.udp.send_to(&buf, config_remote_ip.clone()).await {
                Ok(size) => debug!("Sent packet of size {:?} to {:?}", size, config_remote_ip),
                Err(e) => warn!("Failed to send packet: {:?}", e),
            }
        }
    }

    pub async fn udp_listener(&self) {
        let mut buf = vec![0; TUN_MTU];

        loop {
            let (size, peer) = self.udp.recv_from(&mut buf).await.unwrap();
            if let SocketAddr::V4(peer_addr) = peer {
                debug!("Recieved UDP packet size {:?} from {:?}", size, peer_addr);
                let send_size = self.device.send(&buf).await.unwrap();
                debug!("Send packet with size {:?} to TUN interface", send_size);
            }
        }
    }
}
