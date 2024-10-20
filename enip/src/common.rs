use alloc::vec::Vec;
use nom::IResult;

pub trait Serializable {
    fn deserialize(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized;
    fn serialize(&self) -> Vec<u8>;
}