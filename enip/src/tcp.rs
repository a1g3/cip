use tokio::{io::{self, AsyncReadExt, AsyncWriteExt, Interest}, net::TcpStream};

use crate::{common::Serializable, cpf::{CPFItem, CommonPacketHeader, CommonPacketList, ConnectedAddressItem, ConnectedDataItem, NullAddressItem, UnconnectedDataItem}, encapsulation::{ENIPPacket, EtherNetIPHeader, RegisterSession, SendRRData, SendUnitData, UnregisterSession, NOP}, udp::UdpENIPClient};

pub struct TcpEnipClient {
    pub session_handle: u32,
    connection_id: u32,
    tcp: TcpStream
}

pub struct EnipResult {
    pub status: u32,
    pub data: Vec<u8>,
}

pub enum EnipClient {
    Udp(UdpENIPClient),
    Tcp(TcpEnipClient)
}

impl TcpEnipClient {
    pub fn new(stream: TcpStream) -> Self {
        Self { session_handle: 0, tcp: stream, connection_id: 0 }
    }
    
    pub async fn begin_session(&mut self) {
        let mut header = RegisterSession { header: EtherNetIPHeader { command: 0x0065, length: 4, session_handle: 0, status: 0, sender_context: 0, options: 0}, version: 1, options: 0 };
        let _ = self.send_packet(&mut header).await; 
        let buf = self.read_packet().await;
        let reply = RegisterSession::deserialize(&buf).unwrap();

        self.session_handle = reply.1.header.session_handle;
    }

    pub async fn close_session(&mut self) {
        let mut unreg = UnregisterSession { command: 0x0066, length: 0, session_handle: self.session_handle, status: 0, sender_context: 0, options: 0 };
        let _ = self.send_packet(&mut unreg).await; 
        let _ = self.tcp.shutdown().await;
    }

    pub async fn send_unconnected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x6F, session_handle: self.session_handle, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut items: Vec<Box<dyn CPFItem>> = vec![];
        let null = Box::new(NullAddressItem{ type_id: 0, length: 0 });
        let unconnected = Box::new(UnconnectedDataItem { header: CommonPacketHeader { type_id: 0xb2, length: packet.len() as u16 }, data: packet });
        items.push(null);
        items.push(unconnected);
        let list = CommonPacketList { length: 2, data: items};
        let mut packet = SendRRData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(&mut packet).await;
    }

    pub async fn send_connected(&mut self, packet: Vec<u8>) {
        let header = EtherNetIPHeader { command: 0x70, session_handle: self.session_handle, length: (packet.len() as u16 + 16), status: 0, sender_context: 0, options: 0 };
        let mut items: Vec<Box<dyn CPFItem>> = vec![];
        let address = Box::new(ConnectedAddressItem{ header: CommonPacketHeader { type_id: 0xA1, length: 4 }, addr: self.connection_id  });
        let connected = Box::new(ConnectedDataItem { header: CommonPacketHeader { type_id: 0xB1, length: packet.len() as u16 }, data: packet });
        items.push(address);
        items.push(connected);
        let list = CommonPacketList { length: 2, data: items};
        let mut packet = SendUnitData { header: header, interface_handle: 0, timeout: 0, items: list };
        self.send_packet(&mut packet).await;
    }

    pub async fn send_nop(&mut self) {
        let header = EtherNetIPHeader { command: 0x00, session_handle: self.connection_id, length: 0, status: 0, sender_context: 0, options: 0 };
        let mut packet = NOP { header: header, data: vec![] };
        self.send_packet(&mut packet).await;
    }

    pub async fn send_packet(&mut self, packet: &mut dyn ENIPPacket) {
        let packet: Vec<u8> = packet.serialize().clone();

        let ready = self.tcp.ready(Interest::WRITABLE).await.unwrap();
        if ready.is_writable() {
            match &self.tcp.write_all(&packet).await {
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

    pub async fn read_data(&mut self) -> EnipResult {
        let result = self.read_packet().await;
        let enip = EtherNetIPHeader::deserialize(&result).unwrap();
        let mut data = Vec::new();

        if enip.1.command == 0x006F {
            let rrdata = SendRRData::deserialize(&result).unwrap();
            
            for item in rrdata.1.items.data {
                if item.as_any().is::<UnconnectedDataItem>() {
                    let data_item = item.as_any().downcast_ref::<UnconnectedDataItem>().unwrap();
                    data = data_item.data.clone();
                }
            }
        }

        return EnipResult { status: enip.1.status, data };
    }

    pub async fn read_packet(&mut self) -> Vec<u8> {
        let _ready = self.tcp.readable().await.unwrap();
    
        let mut data: Vec<u8> = vec![0; 65535];
        match self.tcp.read(&mut data).await {
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
}