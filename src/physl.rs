pub mod device;
pub mod repeater;
pub mod byte_host;
pub mod network;

use super::types::*;

pub use device::*;
pub use repeater::*;
pub use byte_host::*;
pub use network::*;

pub fn run_sample() -> Res<()> {
    println!("experimental sample run");
    let mac0 = Mac::new(23);
    let repeater = Box::new(Repeater::new(mac0, "repeater0"));
    let mac1 = Mac::new(24);
    let schedules = vec![
        ByteLog::new(0, Port::new(0), 0x01),
        ByteLog::new(1, Port::new(0), 0x02),
        ByteLog::new(2, Port::new(0), 0x03),
        ByteLog::new(3, Port::new(0), 0x04),
    ];
    let host1 = Box::new(ByteHost::new(mac1, "host1", schedules));
    let mac2 = Mac::new(25);
    let host2 = Box::new(ByteHost::new(mac2, "host2", vec![]));

    let mut nw = Network::new(vec![repeater, host1, host2], vec![]);
    nw.connect_both(mac0, Port::new(0), mac1, Port::new(0))?;
    nw.connect_both(mac0, Port::new(1), mac2, Port::new(0))?;
    nw.run(10)?;

    let d = nw.get_device(mac2)?;
    let d = d.as_any().downcast_ref::<ByteHost>().unwrap();
    println!("{:?}", d.get_rlogs());
    let log = d.get_rlogs();
    assert_eq!(log[0], ByteLog::new(2, Port::new(0), 0x01));
    assert_eq!(log[2], ByteLog::new(4, Port::new(0), 0x03));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2byte_host() {
        run_sample().unwrap();
    }
}
