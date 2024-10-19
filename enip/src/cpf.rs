
use std::any::Any;

use nom::{bytes::streaming::take, number::complete::{be_u32, le_u16, le_u32}, sequence::tuple, IResult, InputTake};

use crate::common::Serializable;


pub trait CPFItem: Serializable {
    fn as_any(&self) -> &dyn Any;
}

pub struct CommonPacketHeader {
    pub type_id: u16,
    pub length: u16
}

impl CPFItem for CommonPacketHeader {
    fn as_any(&self) -> &dyn Any {
        self
    }
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
    pub length: u16,
    pub data: Vec<Box<dyn CPFItem>>
}

impl Serializable for CommonPacketList {
        fn deserialize(input: &[u8]) -> IResult<&[u8], CommonPacketList> {
            let item_count_split = le_u16(input)?;

            let mut remaining_data = item_count_split.0;
            let mut items: Vec<Box<dyn CPFItem>> = Vec::new();
            for _ in 0..item_count_split.1 {
                let item_type = le_u16(remaining_data)?;
                let item_length = le_u16(item_type.0)?;

                if item_length.0.len() < item_length.1.into() {
                    panic!("Not enough data to create Common Packet Item!")
                }

                match item_type.1 {
                    0 =>  {
                        let result: (&[u8], CommonPacketHeader) = NullAddressItem::deserialize(remaining_data)?;
                        items.push(Box::new(result.1));
                        remaining_data = result.0;
                    },
                    0xB2 => { 
                        let result = UnconnectedDataItem::deserialize(remaining_data)?;
                        items.push(Box::new(result.1));
                        remaining_data = result.0;
                    },
                    0xA1 => {
                        let result = ConnectedAddressItem::deserialize(remaining_data)?;
                        items.push(Box::new(result.1));
                        remaining_data = result.0;
                    },
                    0xB1 => {
                        let result = ConnectedDataItem::deserialize(remaining_data)?;
                        items.push(Box::new(result.1));
                        remaining_data = result.0;
                    }
                    _ => panic!("Unknown Common Packet Item")
                }
            }

            return Ok((input, CommonPacketList { length: item_count_split.1, data: items }));
        }
    
        fn serialize(&self) -> Vec<u8> {
            let mut vec = Vec::new();
            vec.extend_from_slice(&self.length.to_le_bytes());
    
            for n in &self.data {
                vec.extend(&n.serialize())
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

impl CPFItem for SockAddrInfo {
    fn as_any(&self) -> &dyn Any {
        self
    }
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

impl CPFItem for ConnectedDataItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
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

impl CPFItem for UnconnectedDataItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
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

impl CPFItem for ConnectedAddressItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
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
    