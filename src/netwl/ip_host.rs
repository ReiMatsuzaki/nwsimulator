use crate::{physl::{BaseDevice, Device}, types::{Res, Mac, UpdateContext}};

use super::{ip_device::{BaseIpDevice, IpDevice}, network_protocol::NetworkProtocol, ip_addr::{IpAddr, SubnetMask}, NetworkLog};

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<NetworkLog>,
    ip_reply_handler: Box<dyn Fn(&Vec<u8>) -> Vec<u8>>,
}

impl IpHost {
    pub fn build_echo(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> Box<IpHost> {
        // let handler = Box::new(
        //     |bytes: &Vec<u8>| Ok(bytes.clone())
        // );
        let base = BaseIpDevice::new(mac, name, vec![ip_addr], 
            subnet_mask);
        let ip_reply_handler = Box::new(
            |bytes: &Vec<u8>| bytes.clone()
        );
        let host = IpHost {
            base,
            schedules: Vec::new(),
            ip_reply_handler,
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
                self.send(p, ctx)?;
            }
        }
        Ok(())
    }
}

impl IpDevice for IpHost {
    fn ip_base(&self) -> &BaseIpDevice {
        &self.base
    }

    fn ip_base_mut(&mut self) -> &mut BaseIpDevice {
        &mut self.base
    }

    fn handle_ip_reply(&mut self, bytes: &Vec<u8>, _ctx: &UpdateContext) -> Res<Option<Vec<u8>>> {
        let bytes = (self.ip_reply_handler)(bytes);
        Ok(Some(bytes))
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
        self.update_from_schedule(ctx)?;
        self.base_update(ctx)?;
        Ok(())
    }
}
