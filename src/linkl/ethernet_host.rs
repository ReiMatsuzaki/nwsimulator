use crate::{types::{Res, Mac}, physl::{Device, BaseDevice, UpdateContext}};

use super::{BaseEthernetDevice, EthernetDevice, EthernetFrame};


pub struct EthernetHost {
    pub base: BaseEthernetDevice,
    handler: Box<dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>>,
}

impl EthernetHost {
    pub fn build_echo(mac: Mac, name: &str) -> Box<EthernetHost> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(Some(bytes.clone()))
        );
        Box::new(Self { base, handler })
    }
}

impl EthernetDevice for EthernetHost {
    fn ether_base(&self) -> &BaseEthernetDevice {
        &self.base
    }

    fn ether_base_mut(&mut self) -> &mut BaseEthernetDevice {
        &mut self.base
    }
}

impl Device for EthernetHost {
    fn base(&self) -> &BaseDevice {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        if let Some(frame) = self.pop_rbuf(ctx) {
            if let Some(bytes) = (self.handler)(&frame.payload)? {
                let frame = EthernetFrame::new(frame.dst, frame.src, frame.ethertype, bytes);
                self.push_sbuf(frame, ctx);
            }
        }
        Ok(())
    }
}

