mod device;
use device::*;

fn main() {
    fn f() -> Res<()> {
        let mut host_a = Device::new_host(0, "HostA");
        let mut host_b = Device::new_host(0, "HostB");
        let mut hub = Device::new_hub(1, "Hub", 2, 2);
        host_a.push_to_send_buf(0, &[1, 2, 3, 4])?;
        for t in 0..4 {
            let x = host_a.send(0)?;
            let x = x.unwrap();
            println!("t:{}, Sent: {}", t, x);
            hub.receive(0, x)?;
            hub.update()?;
        }
        for t in 0..4 {
            let x = hub.send(1)?;
            let x = x.unwrap();
            host_b.receive(0, x)?;
            println!("t:{}, Received: {}", t, x)
        }
        Ok(())
    }
    f().unwrap();
}
