use alloc::vec::Vec;
use nom::{number::complete::{le_u16, le_u32, le_u64}, sequence::tuple, IResult};
use crate::{cpf::CommonPacketList, common::Serializable};

pub trait ENIPPacket: Serializable + Sized {
    fn set_session(&mut self, session_handle: u32);
}

pub struct EtherNetIPHeader {
    pub command: u16,
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub sender_context: u64,
    pub options: u32
}

impl Serializable for EtherNetIPHeader {
    fn deserialize(input: &[u8]) -> IResult<&[u8], EtherNetIPHeader> {
        let (input, (command, length, session_handle, status, sender_context, options)) = tuple((le_u16, le_u16, le_u32, le_u32, le_u64, le_u32))(input)?;

        return Ok((input, EtherNetIPHeader { command, length, session_handle, status, sender_context, options }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.command.to_le_bytes());
        vec.extend_from_slice(&self.length.to_le_bytes());
        vec.extend_from_slice(&self.session_handle.to_le_bytes());
        vec.extend_from_slice(&self.status.to_le_bytes());
        vec.extend_from_slice(&self.sender_context.to_le_bytes());
        vec.extend_from_slice(&self.options.to_le_bytes());

        return vec;  
    }
}

impl ENIPPacket for EtherNetIPHeader {
    fn set_session(&mut self, session_handle: u32) {
        self.session_handle = session_handle;
    }
}

pub type UnregisterSession = EtherNetIPHeader;

pub struct RegisterSession {
    pub header: EtherNetIPHeader,
    pub version: u16,
    pub options: u16
}

impl Serializable for RegisterSession {
    fn deserialize(input: &[u8]) -> IResult<&[u8], RegisterSession> {
        let header = EtherNetIPHeader::deserialize(input)?;
        let (input, (version, options)) = tuple((le_u16, le_u16))(header.0)?;

        return Ok((input, RegisterSession { header: header.1, version, options}))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = self.header.serialize();

        vec.extend_from_slice(&self.version.to_le_bytes());
        vec.extend_from_slice(&self.options.to_le_bytes());

        return vec;  
    }
}

impl ENIPPacket for RegisterSession {
    fn set_session(&mut self, session_handle: u32) {
        self.header.session_handle = session_handle;
    }
}

pub struct SendUnitData {
    pub header: EtherNetIPHeader,
    pub interface_handle: u32,
    pub timeout: u16,
    pub items: CommonPacketList
}

impl Serializable for SendUnitData {
    fn deserialize(input: &[u8]) -> IResult<&[u8], SendUnitData> {
        let header = EtherNetIPHeader::deserialize(input)?;
        let (input, (interface_handle, timeout)) = tuple((le_u32, le_u16))(header.0)?;
        let common_packet_items = CommonPacketList::deserialize(input)?;

        return Ok((common_packet_items.0, SendUnitData { header: header.1, interface_handle, timeout, items: common_packet_items.1 }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = self.header.serialize();

        vec.extend_from_slice(&self.interface_handle.to_le_bytes());
        vec.extend_from_slice(&self.timeout.to_le_bytes());

        if self.items.len() > 0 {
            vec.extend_from_slice(&self.items.len().to_le_bytes());

            vec.extend(self.items.serialize());
        }

        return vec;  
    }
}

impl ENIPPacket for SendUnitData {
    fn set_session(&mut self, session_handle: u32) {
        self.header.session_handle = session_handle;
    }
}

pub struct SendRRData {
    pub header: EtherNetIPHeader,
    pub interface_handle: u32,
    pub timeout: u16,
    pub items: CommonPacketList
}

impl Serializable for SendRRData {
    fn deserialize(input: &[u8]) -> IResult<&[u8], SendRRData> {
        let header = EtherNetIPHeader::deserialize(input)?;
        let (input, (interface_handle, timeout)) = tuple((le_u32, le_u16))(header.0)?;
        let common_packet_items = CommonPacketList::deserialize(input)?;

        return Ok((common_packet_items.0, SendRRData { header: header.1, interface_handle, timeout, items: common_packet_items.1 }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = self.header.serialize();

        vec.extend_from_slice(&self.interface_handle.to_le_bytes());
        vec.extend_from_slice(&self.timeout.to_le_bytes());

        if self.items.len() > 0 {
            vec.extend_from_slice(&self.items.len().to_le_bytes());

            vec.extend(self.items.serialize());
        }

        return vec;  
    }
}

impl ENIPPacket for SendRRData {
    fn set_session(&mut self, session_handle: u32) {
        self.header.session_handle = session_handle;
    }
}

pub struct NOP {
    pub header: EtherNetIPHeader,
    pub data: Vec<u8>
}

impl Serializable for NOP {
    fn deserialize(input: &[u8]) -> IResult<&[u8], NOP> {
        let header = EtherNetIPHeader::deserialize(input)?;

        return Ok((input, NOP { header: header.1, data: header.0.to_vec()}))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = self.header.serialize();

        for n in self.data.clone() {
            vec.push(n)
        }

        return vec;  
    }
}

impl ENIPPacket for NOP {
    fn set_session(&mut self, session_handle: u32) {
        self.header.session_handle = session_handle;
    }
}