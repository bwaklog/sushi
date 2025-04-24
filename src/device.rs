use core::panic;
use std::{net::SocketAddr, process::exit};

use tokio::{net::UdpSocket, sync::Mutex};
use tracing::{debug, error, info, warn};
use tun::{AbstractDevice, AsyncDevice};

pub struct Config {
    pub server: bool,
    pub tun_ip: Option<String>,
    pub udp_local_ip: String,
    pub udp_remote_ip: Option<String>,
}

pub struct VPN {
    pub device: Option<AsyncDevice>,
    udp: UdpSocket,
    config: Config,

    connections: Mutex<Vec<SocketAddr>>,
}

pub const TUN_MTU: usize = 1504;

impl VPN {
    pub async fn init(config: Config) -> VPN {
        let mut device: Option<AsyncDevice>;

        if !config.server && config.tun_ip.is_none() {
            error!("Did not provide a VPN IP for tun interface");
            panic!();
        }

        if !config.server && config.udp_remote_ip.is_none() {
            error!("Did not provide a remote address for the VPN server");
            panic!();
        }

        if !config.server {
            let mut configuration = tun::Configuration::default();
            configuration
                .address(config.tun_ip.as_ref().unwrap().clone())
                .netmask((255, 255, 255, 0))
                .up();

            #[cfg(target_os = "linux")]
            configuration.platform_config(|conf| {
                // requiring root privilege to acquire complete functions
                conf.ensure_root_privileges(true);
            });

            debug!(
                "Created TUN interface with address {:?}",
                config.tun_ip.as_ref().unwrap().clone()
            );

            match tun::create_as_async(&configuration) {
                Ok(dev) => device = Some(dev),
                Err(e) => {
                    error!("failed to create an async device: {:?}", e);
                    exit(1);
                }
            }

            // we encapsulate each packet in a UDP datagram and forward

            // with UDP headers, we have a 28 byte overhead
            // sending with the full TUN MTU size would cause fragmentation
            // so we set the TUN MTU to be lower
            if device.as_mut().unwrap().set_mtu(1472).is_err() {
                error!("failed to set TUN device {:?}", TUN_MTU);
                exit(1);
            }

            debug!(
                "created async TUN device, set MTU to {}",
                device.as_ref().unwrap().mtu().unwrap()
            );
        } else {
            device = None;
        }

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

        VPN {
            device,
            udp,
            config,
            connections: Mutex::new(Vec::new()),
        }
    }

    pub async fn tun_listener(&self) {
        debug!("Listening on TUN interface...");
        let mut buf = vec![0; TUN_MTU];
        let config_remote_ip = self.config.udp_remote_ip.clone();

        loop {
            let size = self.device.as_ref().unwrap().recv(&mut buf).await.unwrap();
            if size > 4 {
                debug!(
                    "Recieved packet with size {:?} from TUN interface, forwarding to {:?}",
                    size, config_remote_ip
                );
            }

            match self
                .udp
                .send_to(&buf, config_remote_ip.as_ref().unwrap().clone())
                .await
            {
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

                let send_size = self.device.as_ref().unwrap().send(&buf).await.unwrap();
                debug!("Send packet with size {:?} to TUN interface", send_size);
            }
        }
    }

    pub async fn vpn_forwarder(&self) {
        let mut buf = vec![0; TUN_MTU];

        loop {
            let (size, peer) = self.udp.recv_from(&mut buf).await.unwrap();

            if let SocketAddr::V4(peer_addr) = peer {
                info!(
                    "Server recieved packet(size {:?}) from {:?}",
                    size, peer_addr
                );
                let mut connections = self.connections.lock().await;

                if !connections.contains(&peer) {
                    connections.push(peer);
                }

                let forward_list = connections.clone().into_iter().filter(|s| *s != peer);

                drop(connections);

                for addr in forward_list {
                    if self.udp.send_to(&buf, addr).await.is_err() {
                        warn!("failed to forward to {:?}", addr);
                    } else {
                        debug!("forwarded packet to {:?}", addr);
                    }
                }
            }
        }
    }
}
