use std::time::Duration;

use cip::{cip::CipClient, common::{self, TcpEnipClient}};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    //let cli = Cli::parse();
    let addr = "192.168.100.53:44818";

    let tcp = TcpStream::connect(addr).await.unwrap();
    tcp.set_nodelay(true).expect("Error setting nodelay");
    let _ = tcp.set_linger(Some(Duration::from_secs(10)));

    let enip_client = TcpEnipClient::new(tcp);
    let mut client = CipClient::new(common::EnipClient::Tcp(enip_client));
    client.connect().await;

    let classes = client.get_supported_classes().await;
    println!("Following objects are implemented {:#04X?}", classes);
}