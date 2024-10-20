use alloc::vec::Vec;
use nom::{bytes::streaming::take, number::complete::{be_u32, le_u16, le_u32}, sequence::tuple, IResult, InputTake};

use crate::common::Serializable;

pub struct CommonPacketHeader {
    pub type_id: u16,
    pub length: u16
}

impl Serializable for CommonPacketHeader {
    fn deserialize(input: &[u8]) -> IResult<&[u8], CommonPacketHeader> {
        let (input, (type_id, length)) = tuple((le_u16, le_u16))(input)?;

        return Ok((input, CommonPacketHeader { type_id, length}))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.type_id.to_le_bytes());
        vec.extend_from_slice(&self.length.to_le_bytes());

        return vec;  
    }
}

pub type NullAddressItem = CommonPacketHeader;

pub struct CommonPacketList
{
    pub null_address_item: Vec<NullAddressItem>,
    pub connected_addr_item: Vec<ConnectedAddressItem>,
    pub connected_data_item: Vec<ConnectedDataItem>,
    pub unconnected_data_item: Vec<UnconnectedDataItem>
}

impl CommonPacketList {
    pub fn new() -> Self {
        Self { connected_addr_item: Vec::new(), connected_data_item: Vec::new(), null_address_item: Vec::new(), unconnected_data_item: Vec::new() }
    }

    pub fn len(&self) -> u16 {
        (self.connected_addr_item.len() + self.connected_data_item.len() + self.unconnected_data_item.len() + self.null_address_item.len() ) as u16
    } 
}

impl Serializable for CommonPacketList {
        fn deserialize(input: &[u8]) -> IResult<&[u8], CommonPacketList> {
            let item_count_split = le_u16(input)?;

            let mut remaining_data = item_count_split.0;
            let mut items = CommonPacketList::new();
            for _ in 0..item_count_split.1 {
                let item_type = le_u16(remaining_data)?;
                let item_length = le_u16(item_type.0)?;

                if item_length.0.len() < item_length.1.into() {
                    panic!("Not enough data to create Common Packet Item!")
                }

                match item_type.1 {
                    0 =>  {
                        let result: (&[u8], CommonPacketHeader) = NullAddressItem::deserialize(remaining_data)?;
                        items.null_address_item.push(result.1);
                        remaining_data = result.0;
                    },
                    0xB2 => { 
                        let result = UnconnectedDataItem::deserialize(remaining_data)?;
                        items.unconnected_data_item.push(result.1);
                        remaining_data = result.0;
                    },
                    0xA1 => {
                        let result = ConnectedAddressItem::deserialize(remaining_data)?;
                        items.connected_addr_item.push(result.1);
                        remaining_data = result.0;
                    },
                    0xB1 => {
                        let result = ConnectedDataItem::deserialize(remaining_data)?;
                        items.connected_data_item.push(result.1);
                        remaining_data = result.0;
                    }
                    _ => panic!("Unknown Common Packet Item")
                }
            }

            return Ok((input, CommonPacketList { connected_addr_item: Vec::new(), null_address_item: Vec::new(), unconnected_data_item: Vec::new(), connected_data_item: Vec::new() }));
        }
    
        fn serialize(&self) -> Vec<u8> {
            let mut vec = Vec::new();
            let count: u16 = self.len();
            vec.extend_from_slice(&count.to_le_bytes());
    
            for item in &self.connected_addr_item {
                vec.extend(item.serialize());
            }

            for item in &self.null_address_item {
                vec.extend(item.serialize());
            }

            for item in &self.connected_data_item {
                vec.extend(item.serialize());
            }

            for item in &self.unconnected_data_item {
                vec.extend(item.serialize());
            }

            return vec;  
        }
    }

pub struct SockAddrInfo {
    pub header: CommonPacketHeader,
    pub sin_family: u32,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8;8],
}

impl Serializable for SockAddrInfo {
    fn deserialize(input: &[u8]) -> IResult<&[u8], SockAddrInfo> {
        let (input, (type_id, length, sin_family, sin_port, sin_addr, sin_zero_context)) = tuple((le_u16, le_u16, be_u32, le_u16, le_u32, take(8u8)))(input)?;
        let sin_zero = sin_zero_context.try_into().expect("slice with incorrect length");

        return Ok((input, SockAddrInfo { header: CommonPacketHeader { type_id, length }, sin_family, sin_port, sin_addr, sin_zero}))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.header.type_id.to_le_bytes());
        vec.extend_from_slice(&self.header.length.to_le_bytes());
        vec.extend_from_slice(&self.sin_family.to_be_bytes());
        vec.extend_from_slice(&self.sin_port.to_be_bytes());
        vec.extend_from_slice(&self.sin_addr.to_be_bytes());

        for n in self.sin_zero {
            vec.push(n)
        }      

        return vec;  
    }
}


pub struct ConnectedDataItem {
    pub header: CommonPacketHeader,
    pub data: Vec<u8>
}

impl Serializable for ConnectedDataItem {
    fn deserialize(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized {
        let (input, (type_id, length)) = tuple((le_u16, le_u16))(input)?;
        let data= input.take(length.into()).to_vec();

        // TODO: Fix input as it still has "data" field when we return
        return Ok((input, ConnectedDataItem { header: CommonPacketHeader { type_id, length }, data }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.header.type_id.to_le_bytes());
        vec.extend_from_slice(&self.header.length.to_le_bytes());

        for n in self.data.iter() {
            vec.push(*n)
        }      

        return vec;  
    }
}

pub struct UnconnectedDataItem {
    pub header: CommonPacketHeader,
    pub data: Vec<u8>
}

impl Serializable for UnconnectedDataItem {
    fn deserialize(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized {
        let (input, (type_id, length)) = tuple((le_u16, le_u16))(input)?;
        let data= input.take(length.into()).to_vec();

        // TODO: Fix input as it still has "data" field when we return
        return Ok((input, UnconnectedDataItem { header: CommonPacketHeader { type_id, length }, data }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.header.type_id.to_le_bytes());
        vec.extend_from_slice(&self.header.length.to_le_bytes());

        for n in self.data.iter() {
            vec.push(*n)
        }      

        return vec;  
    }
}

pub struct ConnectedAddressItem {
    pub header: CommonPacketHeader,
    pub addr: u32
}

impl Serializable for ConnectedAddressItem {
    fn deserialize(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized {
        let (input, (type_id, length, addr)) = tuple((le_u16, le_u16, le_u32))(input)?;

        return Ok((input, ConnectedAddressItem { header: CommonPacketHeader { type_id, length }, addr }))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.header.type_id.to_le_bytes());
        vec.extend_from_slice(&self.header.length.to_le_bytes());
        vec.extend_from_slice(&self.addr.to_le_bytes());

        return vec;  
    }
}
    