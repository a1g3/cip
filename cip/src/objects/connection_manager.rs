use alloc::vec::Vec;
use rand::Rng;

use crate::{cip::{EPath, MessageRouterRequest}, common::Serializable};

pub struct ForwardOpenRequest {
    pub priority: u8,
    pub timeout_ticks: u8,
    pub ot_network_connection_id: u32,
    pub to_network_connection_id: u32,
    pub connection_serial_number: u16,
    pub original_vendor_id: u16,
    pub original_serial_number: u32,
    pub connection_timeout_multiplier: u8,
    pub ot_rpi: u32,
    pub ot_network_parameters: u16,
    pub to_rpi: u32,
    pub to_network_parameters: u16,
    pub transport_class: u8,
    pub connection_path: EPath
}

impl Serializable for ForwardOpenRequest {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.priority);
        vec.push(self.timeout_ticks);
        vec.extend_from_slice(&self.ot_network_connection_id.to_le_bytes());
        vec.extend_from_slice(&self.to_network_connection_id.to_le_bytes());
        vec.extend_from_slice(&self.connection_serial_number.to_le_bytes());
        vec.extend_from_slice(&self.original_vendor_id.to_le_bytes());
        vec.extend_from_slice(&self.original_serial_number.to_le_bytes());
        vec.push(self.connection_timeout_multiplier);
        vec.push(0);vec.push(0);vec.push(0);
        vec.extend_from_slice(&self.ot_rpi.to_le_bytes());
        vec.extend_from_slice(&self.ot_network_parameters.to_le_bytes());
        vec.extend_from_slice(&self.to_rpi.to_le_bytes());
        vec.extend_from_slice(&self.to_network_parameters.to_le_bytes());
        vec.push(self.transport_class);
        

        let mut segments = Vec::new();

        for segement in &self.connection_path.attributes {
            segments.extend(segement.as_ref().serialize());
        }

        if segments.len() % 2 != 0 {
            panic!("Segments are not padded to 16-bit values!");
        }
        vec.push((segments.len() / 2) as u8);
        vec.extend(segments);

        return vec;
    }
}

impl ForwardOpenRequest {
    pub fn create_null_forward_open(path: EPath) -> ForwardOpenRequest {
        let mut rng = rand::thread_rng();

        ForwardOpenRequest { 
            priority: 0, 
            timeout_ticks: 255, 
            ot_network_connection_id: 0, 
            to_network_connection_id: 0, 
            connection_serial_number: rng.gen(), 
            original_vendor_id: 0x1, 
            original_serial_number: 0x12345678, 
            connection_timeout_multiplier: 0, 
            ot_rpi: 0, 
            ot_network_parameters: 0, 
            to_rpi: 0, 
            to_network_parameters: 0, 
            transport_class: 0, 
            connection_path: path
        }
    }
}

pub struct UnconnectedSendRequest {
    pub priority: u8,
    pub timeout_ticks: u8,
    pub message_request: MessageRouterRequest,
    pub route_path: EPath
}

impl Serializable for UnconnectedSendRequest {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.priority);
        vec.push(self.timeout_ticks);
        let embedded_message_request = self.message_request.serialize();
        let length: u16 = embedded_message_request.len() as u16;
        vec.extend_from_slice(&length.to_le_bytes());
        vec.extend(embedded_message_request);
        if length % 2 != 0 {
            vec.push(0);
        }

        let mut segments = Vec::new();

        for segement in &self.route_path.attributes {
            segments.extend(segement.as_ref().serialize());
        }

        if segments.len() % 2 != 0 {
            panic!("Segments are not padded to 16-bit values!");
        }
        vec.push((segments.len() / 2) as u8);
        vec.push(0);
        vec.extend(segments);

        return vec;
    }
}