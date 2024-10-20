use async_trait::async_trait;
use cip::cip::{Client, DataResult};
use tokio::{io::{self, Interest}, net::UdpSocket};

use crate::{common::{Serializable}, cpf::{CommonPacketHeader, CommonPacketList, ConnectedAddressItem, ConnectedDataItem, NullAddressItem, UnconnectedDataItem}, encapsulation::{EtherNetIPHeader, SendRRData, SendUnitData, NOP}};

pub struct UdpENIPClient {
    udp: UdpSocket,
    connection_id: u32
}

impl UdpENIPClient {
    fn new(stream: UdpSocket) -> Self {
        Self { udp: stream, connection_id: 0 }
    }

    async fn read_packet(&self) -> Vec<u8> {
        let _ready = self.udp.readable().await.unwrap();
    
        let mut data: Vec<u8> = vec![0; 65535];

        match self.udp.recv(&mut data).await {
            Ok(n) => {
                if n >= 24 {
                    let mut local = vec![0; n];
                    for i in 0..n {
                        local[i] = data[i]
                    }
                    return local;
                }

                return vec![]
            }
            Err(e) => {
                println!("Error: {}", e);
                return vec![];
            }

        }
    }

    pub async fn send_packet(&mut self, packet: Vec<u8>) {
        let ready = self.udp.ready(Interest::WRITABLE).await.unwrap();
        if ready.is_writable() {
            match &self.udp.send(&packet).await {
                Ok(_) => {
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                }
                Err(_) => {
                    //x.push(work_item).await;
                    println!("ERROR");
                }
            }
        }
    } 
}

#[async_trait]
impl Client for UdpENIPClient {
    async fn begin_session(&mut self) {
        return;
    }

    async fn close_session(&mut self) {
        return;
    }

    async fn send_unconnected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x6F, session_handle: 0, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut list = CommonPacketList::new();
        list.null_address_item.push(NullAddressItem{ type_id: 0, length: 0 });
        list.unconnected_data_item.push(UnconnectedDataItem { header: CommonPacketHeader { type_id: 0xb2, length: packet.len() as u16 }, data: packet });
        let packet = SendRRData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(packet.serialize()).await;
    }

    async fn send_connected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x70, session_handle: 0, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut list = CommonPacketList::new();
        list.connected_addr_item.push(ConnectedAddressItem{ header: CommonPacketHeader { type_id: 0xA1, length: 4 }, addr: self.connection_id  });
        list.connected_data_item.push(ConnectedDataItem { header: CommonPacketHeader { type_id: 0xB1, length: packet.len() as u16 }, data: packet });
        let packet = SendUnitData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(packet.serialize()).await;
    }

    async fn send_nop(&mut self) {
        let header = EtherNetIPHeader { command: 0x00, session_handle: self.connection_id, length: 0, status: 0, sender_context: 0, options: 0 };
        let packet = NOP { header: header, data: vec![] };
        self.send_packet(packet.serialize()).await;
    }

    async fn read_data(&mut self) -> DataResult {
        let result = self.read_packet().await;
        let enip = EtherNetIPHeader::deserialize(&result).unwrap();
        let mut data = Vec::new();

        if enip.1.command == 0x006F {
            let rrdata = SendRRData::deserialize(&result).unwrap();
            
            for item in rrdata.1.items.unconnected_data_item {
                data.extend_from_slice(&item.data);
            }
        } else if enip.1.command == 0x0070 {
            let rrdata= SendUnitData::deserialize(&result).unwrap();
            
            for item in rrdata.1.items.connected_data_item {
                data.extend_from_slice(&item.data);
            }
        }

        return DataResult { status: enip.1.status, data };
    }
}