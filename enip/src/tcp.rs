use alloc::vec::Vec;
use async_trait::async_trait;
use cip::cip::{Client, DataResult};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt, Interest}, net::TcpStream};
use alloc::boxed::Box;
use crate::{common::Serializable, cpf::{CommonPacketHeader, CommonPacketList, ConnectedAddressItem, ConnectedDataItem, NullAddressItem, UnconnectedDataItem}, encapsulation::{EtherNetIPHeader, RegisterSession, SendRRData, SendUnitData, UnregisterSession, NOP}, udp::UdpENIPClient};

pub struct TcpEnipClient {
    pub session_handle: u32,
    connection_id: u32,
    tcp: TcpStream
}

pub enum EnipClient {
    Udp(UdpENIPClient),
    Tcp(TcpEnipClient)
}

impl TcpEnipClient {
    pub fn new(stream: TcpStream) -> Self {
        Self { session_handle: 0, tcp: stream, connection_id: 0 }
    }

    pub async fn send_packet(&mut self, packet: Vec<u8>) {
        let ready = self.tcp.ready(Interest::WRITABLE).await.unwrap();
        if ready.is_writable() {
            match &self.tcp.write_all(&packet).await {
                Ok(_) => {
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                }
                Err(_) => {
                    //x.push(work_item).await;
                }
            }
        }
    } 

    async fn read_packet(&mut self) -> Vec<u8> {
        let _ready = self.tcp.readable().await.unwrap();
    
        let mut data: Vec<u8> = alloc::vec![0; 8192];
        match self.tcp.read(&mut data).await {
            Ok(n) => {
                if n >= 24 {
                    let mut local = alloc::vec![0; n];
                    for i in 0..n {
                        local[i] = data[i]
                    }
                    return local;
                }

                return alloc::vec![]
            }
            Err(e) => {
                return alloc::vec![];
            }

        }
    }
}

#[async_trait]
impl Client for TcpEnipClient {
    async fn begin_session(&mut self) {
        let header = RegisterSession { header: EtherNetIPHeader { command: 0x0065, length: 4, session_handle: 0, status: 0, sender_context: 0, options: 0}, version: 1, options: 0 };
        let _ = self.send_packet(header.serialize()); 
        let buf = self.read_packet().await;
        let reply = RegisterSession::deserialize(&buf).unwrap();

        self.session_handle = reply.1.header.session_handle;
    }

    async fn close_session(&mut self) {
        let unreg = UnregisterSession { command: 0x0066, length: 0, session_handle: self.session_handle, status: 0, sender_context: 0, options: 0 };
        let _ = self.send_packet(unreg.serialize()); 
        let _ = self.tcp.shutdown().await;
    }

    async fn send_unconnected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x6F, session_handle: self.session_handle, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut list: CommonPacketList = CommonPacketList::new();
        list.null_address_item.push(NullAddressItem{ type_id: 0, length: 0 });
        list.unconnected_data_item.push(UnconnectedDataItem { header: CommonPacketHeader { type_id: 0xb2, length: packet.len() as u16 }, data: packet });
        let packet = SendRRData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(packet.serialize()).await;
    }

    async fn send_connected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x70, session_handle: self.session_handle, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut list: CommonPacketList = CommonPacketList::new();
        list.connected_addr_item.push(ConnectedAddressItem{ header: CommonPacketHeader { type_id: 0xA1, length: 4 }, addr: self.connection_id  });
        list.connected_data_item.push(ConnectedDataItem { header: CommonPacketHeader { type_id: 0xB1, length: packet.len() as u16 }, data: packet });
        let packet = SendUnitData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(packet.serialize()).await;
    }

    async fn send_nop(&mut self) {
        let header = EtherNetIPHeader { command: 0x00, session_handle: self.connection_id, length: 0, status: 0, sender_context: 0, options: 0 };
        let packet = NOP { header: header, data: Vec::new() };
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

        }

        return DataResult { status: enip.1.status, data };
    }
}