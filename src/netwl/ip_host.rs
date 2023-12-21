use crate::{physl::{BaseDevice, Device, UpdateContext}, types::{Res, Mac}};

use super::{ip_device::{BaseIpDevice, IpDevice}, network_protocol::NetworkProtocol, ip_addr::{IpAddr, SubnetMask}, NetworkLog};

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<NetworkLog>,
    // handler: Box<dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>>,
}

impl IpHost {
    pub fn build_echo(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> Box<IpHost> {
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(bytes.clone())
        );
        let base = BaseIpDevice::new(mac, name, vec![ip_addr], 
            subnet_mask, handler);
        // let handler = Box::new(
        //     |bytes: &Vec<u8>| Ok(Some(bytes.clone()))
        // );
        let host = IpHost {
            base,
            schedules: Vec::new(),
            // handler,
        };
        Box::new(host)
    }

    pub fn add_schedule(&mut self, t: usize, p: NetworkProtocol) {
        self.schedules.push(NetworkLog { t, p } );
    }

    fn update_from_schedule(&mut self, ctx: &UpdateContext) -> Res<()> {
        for idx in 0..self.schedules.len() {
            let s = &self.schedules[idx];
            if s.t == ctx.t {
                let p = s.p.clone();
                self.push_sbuf(p, ctx)?;
            }
        }
        Ok(())
    }

    fn handle(&mut self, p: &NetworkProtocol) -> Res<Option<NetworkProtocol>> {
        match p {
            NetworkProtocol::IP(ip) => self.base_handle_ip(ip),
            NetworkProtocol::ARP(arp) => self.base_handle_arp(arp),
        }
    }
}

impl IpDevice for IpHost {
    fn ip_base(&self) -> &BaseIpDevice {
        &self.base
    }

    fn ip_base_mut(&mut self) -> &mut BaseIpDevice {
        &mut self.base
    }
}

impl Device for IpHost {
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
        // FIXME: move schedule to IpHost
        self.update_from_schedule(ctx)?;

        while let Some(p) = self.pop_rbuf(ctx)? {
            if let Some(p) = self.handle(&p)? {
                // sbuf.push_back(p);
                self.push_sbuf(p, ctx)?;
            }
        }

        self.update_table()?;

        Ok(())
    }
}
