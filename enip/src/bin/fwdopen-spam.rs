use std::{thread, time::{self, Duration}};

use cip::{cip::{CipClass, CipClient, EPath, LogicalSegment, LogicalType}, common::Serializable, objects::connection_manager::ForwardOpenRequest};
use enip::tcp::TcpEnipClient;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let addr = "192.168.100.2:44818";

    let tcp = TcpStream::connect(addr).await.unwrap();
    tcp.set_nodelay(true).expect("Error setting nodelay");
    let _ = tcp.set_linger(Some(Duration::from_secs(10)));

    let enip_client = TcpEnipClient::new(tcp);
    let mut client = CipClient::new(enip_client);
    client.connect().await;
    let mut response_status = 0;

    while response_status == 0 {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
    
        class_segment.set_segment(LogicalType::ClassId as u8, CipClass::MessageRouter as u32);
        instance_segment.set_segment(LogicalType::InstanceId as u8, 0x1);

        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
        let mut rng = 0x11112222;

        let forward_open = ForwardOpenRequest { 
            priority: 0xF, 
            timeout_ticks: 0xFF, 
            ot_network_connection_id: 0, 
            to_network_connection_id: rng, 
            connection_serial_number: 0, 
            original_vendor_id: 0x011b, 
            original_serial_number: rng, 
            connection_timeout_multiplier: 2, 
            ot_rpi: 50000000, 
            ot_network_parameters: 0x43ff, 
            to_rpi: 50000000, 
            to_network_parameters: 0x43ff, 
            transport_class: 0xA3, 
            connection_path: epath 
        };
    
        let response  = client.call_service(CipClass::ConnectionManager as u32, 0x1, 0x54, forward_open.serialize()).await;
        response_status = response.general_status;
        rng = rng + 1;
    }

    loop {
        client.send_nop().await;

        let one_minute = time::Duration::from_secs(60);
        thread::sleep(one_minute);
        
    }
    
    //client.disconnect().await;

}