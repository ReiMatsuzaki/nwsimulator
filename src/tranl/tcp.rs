use crate::types::*;

#[derive(Debug, PartialEq, Eq)]
pub struct TCP {
    pub src: TPort,
    pub dst: TPort,
    pub seq: u32,
    pub ack: u32,
    data_offset: u8,
    flags: u8,
    window: u16,
    checksum: u16,
    urgent_ptr: u16,
    payload: Vec<u8>,
    pub content: TcpContent,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TcpContent {
    Syn,
    SynAck,
    Ack,
    Fin,
    FinAck,
    Data,
}

impl TCP {
    pub fn new_data(src: TPort, dst: TPort, seq: u32, ack: u32, flags: u8, payload: Vec<u8>) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack,
            data_offset: 0,
            flags,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload,
            content: TcpContent::Data,
        }
    }

    pub fn new_syn(src: TPort, dst: TPort, seq: u32) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack: 0,
            data_offset: 0,
            flags: 0x02,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload: Vec::new(),
            content: TcpContent::Syn,
        }
    }

    pub fn new_ack(src: TPort, dst: TPort, seq: u32, ack: u32) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack,
            data_offset: 0,
            flags: 0x10,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload: Vec::new(),
            content: TcpContent::Ack,
        }
    }

    pub fn new_synack(src: TPort, dst: TPort, seq: u32) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack: 0,
            data_offset: 0,
            flags: 0x12,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload: Vec::new(),
            content: TcpContent::SynAck,
        }
    }

    pub fn new_fin(src: TPort, dst: TPort, seq: u32, ack: u32) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack,
            data_offset: 0,
            flags: 0x01,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload: Vec::new(),
            content: TcpContent::Fin,
        }
    }

    pub fn new_finack(src: TPort, dst: TPort, seq: u32, ack: u32) -> TCP {
        TCP {
            src,
            dst,
            seq,
            ack,
            data_offset: 0,
            flags: 0x11,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            payload: Vec::new(),
            content: TcpContent::FinAck,
        }
    }

    pub fn decode(bytes: &Vec<u8>) -> Res<TCP>  {
        if bytes.len() < 20 {
            return Err(Error::NotEnoughBytes);
        }
        let src = TPort::new(u16::from_be_bytes([bytes[0], bytes[1]]));
        let dst = TPort::new(u16::from_be_bytes([bytes[2], bytes[3]]));
        let seq = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let ack = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let data_offset = bytes[12] >> 4;
        let flags = bytes[13];
        let window = u16::from_be_bytes([bytes[14], bytes[15]]);
        let checksum = u16::from_be_bytes([bytes[16], bytes[17]]);
        let urgent_ptr = u16::from_be_bytes([bytes[18], bytes[19]]);
        let payload = bytes[20..].to_vec();
        let content = match flags {
            0x02 => TcpContent::Syn,
            0x12 => TcpContent::SynAck,
            0x10 => TcpContent::Ack,
            0x01 => TcpContent::Fin,
            0x11 => TcpContent::FinAck,
            _ => TcpContent::Data,
        };
        Ok(TCP {
            src,
            dst,
            seq,
            ack,
            data_offset,
            flags,
            window,
            checksum,
            urgent_ptr,
            payload,
            content,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.src.value.to_be_bytes());
        bytes.extend_from_slice(&self.dst.value.to_be_bytes());
        bytes.extend_from_slice(&self.seq.to_be_bytes());
        bytes.extend_from_slice(&self.ack.to_be_bytes());
        bytes.push(self.data_offset << 4);
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.window.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.urgent_ptr.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp() {
        let tcps = vec![
            TCP::new_data(TPort::new(1), TPort::new(2), 0, 0, 0, vec![1, 2, 3]),
            TCP::new_ack(TPort::new(1), TPort::new(2), 0, 0),
        ];
        for tcp in tcps {
            let bytes = tcp.encode();
            let tcp2 = TCP::decode(&bytes).unwrap();
            assert_eq!(tcp, tcp2);
        }

    }
}