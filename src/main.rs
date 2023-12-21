mod utils;
mod output;
mod types;
mod physl;
mod linkl;
mod netwl;
mod tranl;

use std::io;
use clap::Parser;

#[derive(Clone, Copy)]


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help="run number")]
    rnum: u32,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let rnum = &args.rnum;

    match rnum {
        // 1 => physl::network::run_main().unwrap(),
        // 2 => linkl::ethernet_frame::run_linkl_sample(),
        // 2 => { linkl::run_linkl_sample2().unwrap(); },
        // 3 => { linkl::run_sample_ethernet_switch().unwrap(); } ,

        10 => { physl::run_sample().unwrap(); }
        20 => { linkl::run_sample().unwrap(); }
        21 => { linkl::run_sample_3host().unwrap(); }

        30 => { netwl::run_host_host().unwrap(); }
        31 => { netwl::run_2host_1router().unwrap(); }
        32 => { netwl::run_2router().unwrap(); }
        33 => { netwl::run_unreachable().unwrap(); }
        34 => { netwl::run_test_router_arp().unwrap(); }

        40 => { tranl::run_test_tcp().unwrap(); }
        _ => println!("No such run number"),
    }

    Ok(())
}
