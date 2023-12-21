use crate::{physl::{BaseDevice, Device}, types::{Res, Mac, UpdateContext}};

use super::{ip_device::{BaseIpDevice, IpDevice}, ip_addr::{IpAddr, SubnetMask}};

pub struct Router {
    base: BaseIpDevice,
}

impl Router {
    pub fn build(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask) -> Box<Router> {
        // let handler = Box::new(
        //     |bytes: &Vec<u8>| Ok(bytes.clone())
        // );
        let base = BaseIpDevice::new(mac, name, ip_addr_list, subnet_mask);
        let host = Router {
            base,
        };
        Box::new(host)
    }
}

impl IpDevice for Router {
    fn ip_base(&self) -> &BaseIpDevice {
        &self.base
    }

    fn ip_base_mut(&mut self) -> &mut BaseIpDevice {
        &mut self.base
    }

    fn handle_ip_reply(&mut self, _bytes: &Vec<u8>, _ctx: &UpdateContext) -> Res<Option<Vec<u8>>> {
        panic!("i am router");
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
        self.base_update(ctx)?;
        Ok(())
    }
}
