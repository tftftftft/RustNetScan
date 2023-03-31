use clap::Parser;
use std::net::{Ipv4Addr, TcpStream, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use std::io::ErrorKind;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args{
    #[arg(short, long)]
    range: String,

    #[arg(short, long)]
    ports: u16,
}

fn main(){
    let input = Args::parse();
    

    println!("Input: {}, {}", input.range, input.ports);

    let ip_list: Vec<Ipv4Addr> = cidr_to_ips(input.range);

    println!("List: {:?}", ip_list);

    check_port(ip_list, input.ports);
}


fn cidr_to_ips(cidr: String) -> Vec<Ipv4Addr> {
    let mut ips = Vec::new();

    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() == 2 {
        let base_ip = Ipv4Addr::from_str(parts[0]).unwrap();
        let subnet_mask: u8 = parts[1].parse().unwrap_or(0);

        let base_ip_int = u32::from(base_ip);

        let range = 1 << (32 - subnet_mask);

        // Adjust the loop to start from 1 and end at range - 1
        for i in 1..(range - 1) {
            let new_ip_int = base_ip_int | i;

            let new_ip = Ipv4Addr::from(new_ip_int);
            ips.push(new_ip);
        }
    }

    ips
}

fn check_port(l: Vec<Ipv4Addr>, p: u16) {
    for ip in l {
        let socket = SocketAddr::new(ip.into(), p);
        match TcpStream::connect_timeout(&socket, Duration::from_secs(5)) {
            Ok(_) => println!("Ip {} - alive on port {}", ip, p),
            Err(e) => match e.kind() {
                ErrorKind::TimedOut => println!("Ip {} timed out on port {}", ip, p),
                _ => println!("Ip {} is dead on port {} - Error: {}", ip, p, e),
            },
        }
    }
}