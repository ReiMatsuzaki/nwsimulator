use std::collections::VecDeque;

use crate::physl::{device::{DeviceOperation, DeviceContext}, physl_error::PhyslError};

use super::{ethernet_frame::EthernetFrame, linkl_error::{LinklError, Res}};

pub trait EthernetOperation {
    // event handler for ethernet frame arrive
    fn apply(&mut self,
             ctx: &DeviceContext, 
             rbufs: &mut Vec<VecDeque<EthernetFrame>>,
             sbufs: &mut Vec<VecDeque<EthernetFrame>>,
            ) -> Res<()>;
    // event handller for time tick
    // fn update(&mut self, ctx: &DeviceContext) -> Res<Vec<(usize, EthernetFrame)>>;
}

pub struct Ethernet {
    pub op: Box<dyn EthernetOperation>,
    rbufs: Vec<VecDeque<EthernetFrame>>,
    sbufs: Vec<VecDeque<EthernetFrame>>,
}

impl DeviceOperation for Ethernet {
    fn apply(&mut self, 
             ctx: &DeviceContext, rbufs: &mut Vec<Vec<u8>>, sbufs: &mut Vec<VecDeque<u8>>) -> Result<(), PhyslError> {
        let disp = crate::output::is_frame_level();
        
        // push to ethernet receive buffer
        for (port, rbuf) in rbufs.iter_mut().enumerate() {
            match EthernetFrame::decode(rbuf) {
                Ok(frame) => {
                    rbuf.clear();
                    self.rbufs[port].push_back(frame);
                },
                Err(LinklError::NotEnoughBytes) => (), // not enough bytes. do nothing
                Err(e) => return Err(PhyslError::LinklError {e}),
            }
        }

        // receive buffer -> send buffer
        // FIXME: avoid clone
        let mut en_rbufs = self.rbufs.clone();
        let mut en_sbufs = self.sbufs.clone();
        self.op.apply(ctx, &mut en_rbufs, &mut en_sbufs)
            .map_err(|e| PhyslError::LinklError {e})?;
        self.rbufs = en_rbufs;
        self.sbufs = en_sbufs;
        if disp {
            // println!("ethernet");
        }

        // ethernet send buffer -> byte send buffer
        for port in 0..ctx.num_ports {
            let sbuf = &mut self.sbufs[port];
            while let Some(frame) = sbuf.pop_front() {
                let bytes = EthernetFrame::encode(&frame);
                for byte in bytes {
                    sbufs[port].push_back(byte);
                }
            }
        }

        Ok(())
    }
}

impl Ethernet {
    pub fn new(op: Box<dyn EthernetOperation>, num_ports: usize) -> Ethernet {
        Ethernet {
            op,
            rbufs: vec![VecDeque::new(); num_ports],
            sbufs: vec![VecDeque::new(); num_ports],
        }
    }
}
    // fn update(&mut self, ctx: &DeviceContext) -> Result<Vec<(usize, Vec<u8>)>, PhyslError> {
    //     let disp = crate::output::is_frame_level();
    //     let out_frames = self.op.update(ctx)
    //         .map_err(|e| PhyslError::LinklError {e})?;
    //     let out_bytes = out_frames.iter().map(|(port, frame)| {
    //         if disp {
    //             let msg = format!("{:>3},   {:<11}", ctx.t, ctx.name);
    //             println!("{} send to {}. {:?}", msg, port, frame);
    //         }
    //         (port.clone(), EthernetFrame::encode(frame))
    //     }).collect();
    //     Ok(out_bytes)
    // }
// }
