use crate::netwl::IpAddr;

use super::super::types::*;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    // NoConnection,
    Listening,
    SynSent,
    SynAckSent,
    Established,
    DataReceiving,
    DataSent,
    FinSent,
    FinAckSent,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inst {
    Connect(u32, IpAddr, TPort), // client
    Listen(u32, TPort), // server
    Close(u32),
    Send(u32, String),
    Recv(u32),
}

pub struct Socket {
    pub state: State,
    pub dst_port: TPort,
    pub dst_ip: IpAddr,
}

impl Socket {
    pub fn new(state: State, dst_port: TPort, dst_ip: IpAddr) -> Socket {
        Socket {
            state,
            dst_port,
            dst_ip,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}