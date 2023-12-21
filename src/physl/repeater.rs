use super::super::types::*;
use super::device::*;
use std::any::Any;

pub struct Repeater {
    base: BaseDevice,
}

impl Repeater {
    pub fn new(mac: Mac, name: &str) -> Repeater {
        Repeater {
            base: BaseDevice::new(mac, name, 2),
        }
    }
}

impl Device for Repeater {
    fn base(&self) -> &BaseDevice {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()> {
        while let Some((p, x)) = self.base.pop_rbuf() {
            self.base.push_sbuf((Port::new(1 - p.value), x));
        }
        Ok(())
    }
}
