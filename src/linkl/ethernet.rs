use crate::physl::{device::{DeviceOperation, DeviceContext}, physl_error::PhyslError};

use super::{ethernet_frame::EthernetFrame, linkl_error::{LinklError, Res}};

pub trait EthernetOperation {
    fn apply(&mut self, ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>>;
}

pub struct Ethernet {
    pub op: Box<dyn EthernetOperation>,
}
impl DeviceOperation for Ethernet {
    fn apply(&mut self, ctx: &DeviceContext, src_port: usize, rbuf: &Vec<u8>) -> Result<Vec<(usize, Vec<u8>)>, PhyslError> {
        // FIXME: if error occured, receive buffer shold be cleared
        let disp = crate::output::is_frame_level();
        match EthernetFrame::decode(rbuf) {
            Ok(frame) => {
                let msg = format!("{:>3}, {:>8}:{:<2}", ctx.t, ctx.name, src_port);
                if disp {
                    println!("{} receive.   {:?}", msg, frame);
                }
                let out_frames = self.op.apply(ctx, src_port, frame)
                    .map_err(|e| PhyslError::LinklError {e})?;
                let out_bytes = out_frames.iter().map(|(port, frame)| {
                    if disp {
                        println!("{} send to {}. {:?}", " ".repeat(msg.len()), port, frame);
                    }
                    (port.clone(), EthernetFrame::encode(frame))
                }).collect();
                Ok(out_bytes)
            },
            Err(LinklError::NotEnoughBytes) => Ok(vec![]), // not enough bytes
            Err(e) => Err(
                PhyslError::LinklError{e}
            )
        }
    }
}
