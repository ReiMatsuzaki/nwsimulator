pub mod types;
pub mod tcp;
pub mod tcp_host;

pub use types::*;
pub use tcp::*;
pub use tcp_host::*;

use crate::{netwl::{IpAddr, SubnetMask}, physl::Network};

use super::types::*;

pub fn run_test_tcp_nw() -> Res<()> {
    crate::output::set_level(crate::output::Level::Transport);
    let mac0 = Mac::new(721);
    let mac1 = Mac::new(722);
    let ip0 = IpAddr::new(7621);
    let ip1 = IpAddr::new(7622);
    let subnetmask = SubnetMask::new(24);
    let mut host_a = TcpHost::build(mac0, "host_a", ip0, subnetmask);
    let mut host_b = TcpHost::build(mac1, "host_b", ip1, subnetmask);

    host_a.add_arp_entry(ip1, mac1)?;
    host_b.add_arp_entry(ip0, mac0)?;

    host_a.add_inst(Inst::Socket(0));
    host_a.add_inst(Inst::Connect(0, ip1, TPort::new(0)));
    host_a.add_inst(Inst::Send(0, "hello".to_string()));
    host_a.add_inst(Inst::Close(0));

    host_b.add_inst(Inst::Socket(0));
    host_b.add_inst(Inst::Listen(0, TPort::new(0)));
    host_b.add_inst(Inst::Recv(0));
    // host_b.add_inst(Inst::Close(0));

    let mut nw = Network::new(
        vec![host_a, host_b],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac1, Port::new(0))?;
    nw.run(750).unwrap();
    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<TcpHost>().unwrap();
    let rlog = d.get_recv_log();
    assert_eq!(3, rlog.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_nw() {
        run_test_tcp_nw().unwrap();
    }
}