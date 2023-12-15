mod output;
mod physl;
mod linkl;


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
        1 => physl::network::run_main().unwrap(),
        // 2 => linkl::ethernet_frame::run_linkl_sample(),
        2 => { linkl::run_linkl_sample2().unwrap(); },
        3 => { linkl::run_sample_ethernet_switch().unwrap(); } ,
        _ => println!("No such run number"),
    }

    Ok(())
}
