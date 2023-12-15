mod physl;
mod linkl;

fn main() {
    physl::network::run_main().unwrap();
    println!("");
    linkl::ethernet_frame::run_linkl_sample();
    println!("");
    linkl::run_linkl_sample2().unwrap();
}
