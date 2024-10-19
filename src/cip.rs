use nom::{bytes::complete::take, sequence::tuple, InputTake};

use crate::{common::{EnipClient, NetworkSerializable}, objects::{connection_manager::UnconnectedSendRequest, message_router::MessageRouter}};

pub trait EpathSegments: NetworkSerializable {
    fn get_type(&self) -> u8;
    fn get_data(&self) -> Vec<u8>;
}

pub struct LogicalSegment {
    logical_type: u8,
    logical_format: u8,
    value: u32,
}

impl LogicalSegment {
    pub fn new() -> Self {
        Self { logical_format: 0, logical_type: 0, value: 0 }
    }

    pub fn init(logical_type: u8, value: u32) -> Self {
        let mut obj = Self { logical_format: 0, logical_type: 0, value: 0 };
        obj.set_segment(logical_type, value);
        return obj;
    }

    pub fn set_segment(&mut self, logical_type: u8, value: u32) {
        if logical_type > LogicalType::ExdendedLogical as u8 {
            panic!("Cannot send logical type greater than 8")
        }

        if value < 256 {
            self.logical_format = LogicalFormat::EightBit as u8;
        } else if value < 65536 {
            self.logical_format = LogicalFormat::SixteenBit as u8;
        } else {
            self.logical_format = LogicalFormat::ThirtyTwoBit as u8;
        }

        self.logical_type = logical_type;
        self.value = value;
    }
}

impl EpathSegments for LogicalSegment {
    fn get_type(&self) -> u8 {
        let mut result: u8 = 0;
        result = result | 0b00100000;
        result = result | (self.logical_type << 2);
        result = result | self.logical_format;
        
        return result;
    }

    fn get_data(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        match self.logical_format {
            0b00 => result.push(self.value as u8),
            0b01 => result.extend(u16::to_le_bytes(self.value as u16)),
            0b10 => result.extend(u32::to_le_bytes(self.value as u32)),
            _ => panic!("Unknown logical format!")
        }

        return result;
    }
}

impl NetworkSerializable for LogicalSegment {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        let encoding1 = input.take_split(1);
        let encoding = encoding1.0[0];
        let logical_segment_type = (encoding & 0b00011100) >> 2;
        let logical_format = encoding & 0b00000011;

        let value = encoding1.1.take_split((logical_format + 1).into());
        let mut actual_value: u32 = 0;

        for num in value.0 {
            actual_value = (actual_value << 8) + u32::from(num.clone());
        }

        return Ok((value.1, LogicalSegment { logical_type: logical_segment_type, logical_format: logical_format, value: actual_value}));
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        let data = self.get_data();
        vec.push(self.get_type());
        if self.logical_format != LogicalFormat::EightBit as u8 {
            vec.push(0);
        }
        vec.extend(data);

        return vec;
    }
}

pub struct PortSegment {
    pub extended_link_address: bool,
    pub port_identifier: u8,
    pub link_address: Vec<u8>
}

impl EpathSegments for PortSegment {
    fn get_type(&self) -> u8 {
        let mut result: u8 = 0;
        if self.extended_link_address {
            result = result | 0b00010000;
            panic!("Not supported!")
        }
        if self.port_identifier < 15 {
            result = result | self.port_identifier;
        } else {
            todo!("Not supported!")
        }

        return result;
    }

    fn get_data(&self) -> Vec<u8> {
        return self.link_address.clone();
    }
}

impl NetworkSerializable for PortSegment {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        let encoding1 = input.take_split(1);
        let encoding = encoding1.0[0];
        let is_extended = (encoding & 0b00010000) != 0;
        let value = encoding & 0b1111;
        let (reamining, link_address) = input.take_split(1);

        return Ok((reamining, PortSegment { extended_link_address: is_extended, port_identifier: value, link_address: vec![link_address[0]] }));
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.get_type());
        vec.extend(self.get_data());

        return vec;
    }
}

impl PortSegment {
    pub fn new() -> Self {
        Self { port_identifier: 1, extended_link_address: false, link_address: Vec::new() }
    }

    pub fn set_segment(&mut self, port_identifier: u8) {
        if self.port_identifier > 14 {
            todo!("Not supported!");
        }

        self.port_identifier = port_identifier;
    }

    pub fn set_address(&mut self, address: Vec<u8>) {
        self.link_address = address;
    }
}

pub struct EPath {
    pub attributes: Vec<Box<dyn EpathSegments>>
}

impl EPath {
    pub fn new() -> Self {
        Self { attributes: Vec::new() }
    }
}

#[repr(u8)]
#[allow(dead_code)]
pub enum CipService {
    GetAttributesAll = 0x01,
    SetAttributesAll = 0x02,
    GetAttributeList = 0x03,
    SetAttributesList = 0x04,
    Reset = 0x05,
    Start = 0x06,
    Stop = 0x07,
    Create = 0x08,
    Delete = 0x09,
    MultipleServicePacket = 0x0A,
    ApplyAttributes = 0x0D,
    GetAttributeSingle = 0x0E,
    SetAttributeSingle = 0x10,
    Save = 0x16
}

#[repr(u8)]
#[allow(dead_code)]
pub enum  LogicalType {
    ClassId = 0b000,
    InstanceId = 0b001,
    MemberId = 0b010,
    ConnectionPoint = 0b011,
    AttributeId = 0b100,
    Special = 0b101,
    ServiceId = 0b110,
    ExdendedLogical = 0b111
}

#[repr(u8)]
#[allow(dead_code)]
pub enum  LogicalFormat {
    EightBit = 0b00,
    SixteenBit = 0b01,
    ThirtyTwoBit = 0b10,
}

#[repr(u16)]
#[allow(dead_code)]
pub enum CipClass {
    Identity = 0x01,
    MessageRouter = 0x02,
    DeviceNet = 0x03,
    Assembly = 0x04,
    Connection = 0x05,
    ConnectionManager = 0x06,
    Register = 0x07,
    DiscreteInputPoint = 0x08,
    DiscreteOutputPoint = 0x09,
    AnalogInputPoint = 0x0A,
    AnalogOutputPoint = 0x0B,
    Parameter = 0x0F,
    ParameterGroup = 0x10,
    Group = 0x12,
    DiscreteInputGroup = 0x1D,
    DiscreteOutputGroup = 0x1E,
    DiscreteGroup = 0x1F,
    File = 0x37,
    ControlNet = 0xF0,
    ControlNetKeeper = 0xF1,
    ControlNetScheduling = 0xF2,
    ConnectionConfiguration = 0xF3,
    Port = 0xF4,
    TCPIPObject = 0xF5,
    EtherNetLink = 0xF6,
}

#[repr(u16)]
#[allow(dead_code)]
pub enum CipDataType {
    Bool = 0x01,
    Sint = 0x02,
    Int = 0x03,
    Dint = 0x04,
    Lint = 0x05,
    Usint = 0x06,
    Uint = 0x07,
    Udint = 0x08,
    Ulint = 0x09,
    Real = 0x0A,
    Lreal = 0x0B,
    Byte = 0x0C,
    Word = 0x0D,
    Dword = 0x0E,
    Lword = 0x0F,
}

pub struct MessageRouterRequest {
    pub service: u8,
    pub epath: EPath,
    pub data: Vec<u8>
}

impl NetworkSerializable for MessageRouterRequest {
    fn deserialize(_input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let mut segments = Vec::new();

        result.push(self.service);
        
        for segement in &self.epath.attributes {
            segments.extend(segement.as_ref().serialize());
        }

        if segments.len() % 2 != 0 {
            panic!("Segments are not padded to 16-bit values!");
        }
        result.push((segments.len() / 2) as u8);
        result.extend(segments);

        if self.data.len() > 0 {
            result.extend(&self.data)
        }

        return result;
    }
}

pub struct MessageRouterResponse {
    pub service: u8,
    pub reserved: u8,
    pub general_status: u8,
    pub size_of_additional_status: u8,
    pub additional_status: Vec<u16>,
    pub data: Vec<u8>
}

impl NetworkSerializable for MessageRouterResponse {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        let (input, (raw_service, raw_reserved, raw_general_status, raw_size_of_additional_status)) = tuple((take(1u8),take(1u8),take(1u8), take(1u8)))(input)?;
        let service = raw_service[0].into();
        let reserved = raw_reserved[0].into();
        let general_status = raw_general_status[0].into();
        let size_of_additional_status = raw_size_of_additional_status[0].into();

        return Ok((&input, MessageRouterResponse { service, reserved, general_status, size_of_additional_status, additional_status: Vec::new(), data: input.to_vec()}));
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct CipClient {
    enip: EnipClient
}

impl CipClient {
    pub fn new(client: EnipClient) -> Self {
        Self { enip: client }
    }

    pub async fn connect(&mut self) {
        match self.enip {
            EnipClient::Udp(_) => return,
            EnipClient::Tcp(ref mut tcp) => tcp.begin_session().await,
        }
    }

    pub async fn send_unconnected(&mut self, packet: Vec<u8>) {
        match self.enip {
            EnipClient::Udp(ref mut udp) => udp.send_unconnected(packet).await,
            EnipClient::Tcp(ref mut tcp) => tcp.send_unconnected(packet).await,
        }
    }

    pub async fn read_data(&mut self) -> Vec<u8> {
        match self.enip {
            EnipClient::Udp(ref mut udp) => udp.read_data().await.data,
            EnipClient::Tcp(ref mut tcp) => tcp.read_data().await.data,
        }
    }

    pub async fn disconnect(&mut self) {
        match self.enip {
            EnipClient::Udp(ref mut udp) => udp.close_session().await,
            EnipClient::Tcp(ref mut tcp) => tcp.close_session().await,
        }
    }

    pub async fn call_service(&mut self, class_id: u32, instance_id: u32, service_num: u8, data: Vec<u8>) -> MessageRouterResponse {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
    
        let new_service_num = service_num;// & 0b01111111;
    
        class_segment.set_segment(LogicalType::ClassId as u8, class_id);
        instance_segment.set_segment(LogicalType::InstanceId as u8, instance_id);
    
        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
    
        let request = MessageRouterRequest { service: new_service_num, epath, data };

        self.send_unconnected_cm(request).await;

        let data = match self.enip {
            EnipClient::Udp(ref mut udp) => udp.read_data().await,
            EnipClient::Tcp(ref mut tcp) => tcp.read_data().await,
        };
    
        let result = MessageRouterResponse::deserialize(&data.data).unwrap();
        return result.1;
    }

    pub async fn send_unconnected_cm(&mut self, request: MessageRouterRequest) {
        let mut unconnected_send_epath  = EPath::new();
        let mut port_segment = PortSegment::new();
        port_segment.set_address(vec![2]);
        unconnected_send_epath.attributes.push(Box::new(port_segment));
    
        let mut epath  = EPath::new();
        let connection_manager_class = LogicalSegment::init(LogicalType::ClassId as u8, CipClass::ConnectionManager as u32);
        let connection_manager_instance = LogicalSegment::init(LogicalType::InstanceId as u8, 0x1);
        epath.attributes.push(Box::new(connection_manager_class));
        epath.attributes.push(Box::new(connection_manager_instance));
    
        let mut identity_class_segment = LogicalSegment::new();
        let mut identity_instance_segment = LogicalSegment::new(); 
    
        identity_class_segment.set_segment(LogicalType::ClassId as u8, CipClass::Identity as u32);
        identity_instance_segment.set_segment(LogicalType::InstanceId as u8, 1);
    
        let mut identity_epath  = EPath::new();
        identity_epath.attributes.push(Box::new(identity_class_segment));
        identity_epath.attributes.push(Box::new(identity_instance_segment));
    
        let request = MessageRouterRequest { service: 0x52, epath, data: UnconnectedSendRequest { priority: 0b11, timeout_ticks: 240, message_request: request, route_path: unconnected_send_epath }.serialize() };
    
        match self.enip {
            EnipClient::Udp(ref mut udp) => udp.send_unconnected(request.serialize()).await,
            EnipClient::Tcp(ref mut tcp) => tcp.send_unconnected(request.serialize()).await,
        };
    }

    pub async fn get_supported_classes(&mut self) -> Vec<u16> {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
        let mut attribute_segment = LogicalSegment::new(); 
    
        class_segment.set_segment(LogicalType::ClassId as u8, CipClass::MessageRouter as u32);
        instance_segment.set_segment(LogicalType::InstanceId as u8, 1);
        attribute_segment.set_segment(LogicalType::AttributeId as u8, 1);
    
        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
        epath.attributes.push(Box::new(attribute_segment));
    
        let request = MessageRouterRequest { service: CipService::GetAttributesAll as u8, epath, data: vec![] };
        self.send_unconnected(request.serialize()).await;
        let data = match self.enip {
            EnipClient::Udp(ref mut udp) => udp.read_data().await,
            EnipClient::Tcp(ref mut tcp) => tcp.read_data().await,
        };
    
        let response = MessageRouterResponse::deserialize(&data.data).unwrap();
        let get_all_response = MessageRouter::deserialize(&response.1.data).unwrap();
    
        return get_all_response.1.objects;
    }
    
    pub async fn get_attribute_single(&mut self, class_id: u32, instance_id: u32, attribute_id: u32) -> MessageRouterResponse {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
        let mut attribute_segment = LogicalSegment::new(); 
    
        class_segment.set_segment(LogicalType::ClassId as u8, class_id);
        instance_segment.set_segment(LogicalType::InstanceId as u8, instance_id);
        attribute_segment.set_segment(LogicalType::AttributeId as u8, attribute_id);
    
        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
        epath.attributes.push(Box::new(attribute_segment));
    
        let request = MessageRouterRequest { service: CipService::GetAttributeSingle as u8, epath, data: vec![] };
        self.send_unconnected_cm(request).await;
        let data = match self.enip {
            EnipClient::Udp(ref mut udp) => udp.read_data().await,
            EnipClient::Tcp(ref mut tcp) => tcp.read_data().await,
        };
    
        let result = MessageRouterResponse::deserialize(&data.data).unwrap();
        return result.1;
    }
    
    pub async fn set_attribute_single(&mut self, class_id: u32, instance_id: u32, attribute_id: u32) -> MessageRouterResponse {
        let mut class_segment = LogicalSegment::new();
        let mut instance_segment = LogicalSegment::new(); 
        let mut attribute_segment = LogicalSegment::new(); 
    
        class_segment.set_segment(LogicalType::ClassId as u8, class_id);
        instance_segment.set_segment(LogicalType::InstanceId as u8, instance_id);
        attribute_segment.set_segment(LogicalType::AttributeId as u8, attribute_id);
    
        let mut epath  = EPath::new();
        epath.attributes.push(Box::new(class_segment));
        epath.attributes.push(Box::new(instance_segment));
        epath.attributes.push(Box::new(attribute_segment));
    
        let request = MessageRouterRequest { service: CipService::SetAttributeSingle as u8, epath, data: vec![] };
        self.send_unconnected_cm(request).await;
        let data = match self.enip {
            EnipClient::Udp(ref mut udp) => udp.read_data().await,
            EnipClient::Tcp(ref mut tcp) => tcp.read_data().await,
        };
    
        let result = MessageRouterResponse::deserialize(&data.data).unwrap();
        return result.1;
    }

    pub async fn send_nop(&mut self) {
        let data = match self.enip {
            EnipClient::Udp(ref mut udp) => udp.send_nop().await,
            EnipClient::Tcp(ref mut tcp) => tcp.send_nop().await,
        };
    }
}