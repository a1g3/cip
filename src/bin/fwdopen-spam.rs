use std::time::Duration;

use cip::{cip::{CipClass, CipClient, EPath, LogicalSegment, LogicalType}, common::{self, NetworkSerializable, TcpEnipClient}, objects::connection_manager::ForwardOpenRequest};
use rand::Rng;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let addr = "192.168.100.61:44818";

    let tcp = TcpStream::connect(addr).await.unwrap();
    tcp.set_nodelay(true).expect("Error setting nodelay");
    let _ = tcp.set_linger(Some(Duration::from_secs(10)));

    let enip_client = TcpEnipClient::new(tcp);
    let mut client = CipClient::new(common::EnipClient::Tcp(enip_client));
    client.connect().await;

    for _ in 0..35 {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
    
        class_segment.set_segment(LogicalType::ClassId as u8, CipClass::MessageRouter as u32);
        instance_segment.set_segment(LogicalType::InstanceId as u8, 0x1);

        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
        let mut rng = rand::thread_rng();

        let forward_open = ForwardOpenRequest { 
            priority: 0xF, 
            timeout_ticks: 0xFF, 
            ot_network_connection_id: 0, 
            to_network_connection_id: rng.gen(), 
            connection_serial_number: 0, 
            original_vendor_id: 0x011b, 
            original_serial_number: rng.gen(), 
            connection_timeout_multiplier: 2, 
            ot_rpi: 50000000, 
            ot_network_parameters: 0x43ff, 
            to_rpi: 50000000, 
            to_network_parameters: 0x43ff, 
            transport_class: 0xA3, 
            connection_path: epath 
        };
    
        client.call_service(CipClass::ConnectionManager as u32, 0x1, 0x54, forward_open.serialize()).await;

    }

    loop {

    }
    
    //client.disconnect().await;

}