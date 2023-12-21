use crate::types::{Mac, Res};
use crate::physl::{BaseDevice, Device, UpdateContext};

use super::BaseEthernetDevice;

pub struct EthernetSwitch {
    base: BaseEthernetDevice,
}

impl EthernetSwitch {
    fn new(base: BaseEthernetDevice) -> EthernetSwitch {
        EthernetSwitch {
            base,
        }
    }

    pub fn build_bridge(mac: Mac, name: &str) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, 2);
        Box::new(Self::new(base))
    }

    pub fn build_switch(mac: Mac, name: &str, num_ports: usize) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, num_ports);
        Box::new(Self::new(base))
    }
}

impl Device for EthernetSwitch {
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
        // rbuf -> sbuf
        while let Some(frame) = self.base.pop_rbuf(ctx) {
            self.base.push_sbuf(frame, ctx);
        }
        Ok(())
    }
}

