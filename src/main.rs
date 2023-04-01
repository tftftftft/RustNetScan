use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::net::{Ipv4Addr, TcpStream, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use std::io::ErrorKind;
use std::sync::Arc;
use rayon::ThreadPoolBuilder;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args{
    #[arg(short, long)]
    range: String,

    #[arg(short, long, use_value_delimiter = true)]
    ports: Vec<u16>,

    #[arg(short, long, default_value = "4")]
    threads: usize,
}

fn main(){
    let input = Args::parse();
    
    println!("Input: {}, {:?}", input.range, input.ports);

    let ip_list: Vec<Ipv4Addr> = cidr_to_ips(input.range);

    println!("List: {:?}", ip_list);

    check_port(ip_list, input.ports, input.threads);
}


fn cidr_to_ips(cidr: String) -> Vec<Ipv4Addr> {
    let mut ips = Vec::new();

    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() == 2 {
        let base_ip = Ipv4Addr::from_str(parts[0]).unwrap();
        let subnet_mask: u8 = parts[1].parse().unwrap_or(0);

        let base_ip_int = u32::from(base_ip);

        if subnet_mask == 32 {
            // If the subnet mask is /32, add only the single IP address
            ips.push(base_ip);
        } else if subnet_mask < 32 {
            let range = 1 << (32 - subnet_mask);

            // Calculate start and end of the loop based on the subnet mask
            let start = if subnet_mask > 0 { 1 } else { 0 };
            let end = if subnet_mask > 0 { range - 1 } else { range };

            for i in start..end {
                let new_ip_int = base_ip_int | i;

                let new_ip = Ipv4Addr::from(new_ip_int);
                ips.push(new_ip);
            }
        } else {
            // If the subnet mask is invalid (greater than 32), return an empty vector
            eprintln!("Invalid subnet mask: {}", subnet_mask);
        }
    } else {
        // If the user input is not in the correct format, return an empty vector
        eprintln!("Invalid CIDR format: {}", cidr);
    }

    ips
}


fn check_port(l: Vec<Ipv4Addr>, p: Vec<u16>, num_threads: usize) {
    // Wrap the ports vector in an Arc
    let ports = Arc::new(p);

    // Create a thread pool with the specified number of threads
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    // Use the thread pool to process IP addresses in parallel
    pool.install(|| {
        l.par_iter().for_each(|ip| {
            for port in &*ports {
                let socket = SocketAddr::new((*ip).into(), *port);
                match TcpStream::connect_timeout(&socket, Duration::from_secs(5)) {
                    Ok(_) => println!("Ip {} - alive on port {}", ip, port),
                    Err(e) => match e.kind() {
                        ErrorKind::TimedOut => println!("Ip {} timed out on port {}", ip, port),
                        _ => println!("Ip {} is dead on port {} - Error: {}", ip, port, e),
                    },
                }
            }
        });
    });
}