use crate::{physl::{BaseDevice, Device, UpdateContext}, types::{Res, Mac}};

use super::{ip_device::{BaseIpDevice, IpDevice}, network_protocol::NetworkProtocol, ip_addr::{IpAddr, SubnetMask}};

pub struct Router {
    base: BaseIpDevice,
}

impl Router {
    pub fn build(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask) -> Box<Router> {
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(bytes.clone())
        );
        let base = BaseIpDevice::new(mac, name, ip_addr_list, 
            subnet_mask, handler);
        let host = Router {
            base,
        };
        Box::new(host)
    }

    fn handle(&mut self, p: &NetworkProtocol) -> Res<Option<NetworkProtocol>> {
        match p {
            NetworkProtocol::IP(ip) => self.base_handle_ip(ip),
            NetworkProtocol::ARP(arp) => self.base_handle_arp(arp),
        }
    }
}

impl IpDevice for Router {
    fn ip_base(&self) -> &BaseIpDevice {
        &self.base
    }

    fn ip_base_mut(&mut self) -> &mut BaseIpDevice {
        &mut self.base
    }
}

impl Device for Router {
    fn base(&self) -> &BaseDevice {
        self.base.base()
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        self.base.base_mut()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        while let Some(p) = self.pop_rbuf(ctx)? {
            if let Some(p) = self.handle(&p)? {
                self.push_sbuf(p, ctx)?;
            }
        }

        self.update_table()?;

        Ok(())
    }
}
