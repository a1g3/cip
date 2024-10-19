use nom::IResult;

pub trait Serializable {
    fn deserialize(input: &[u8]) -> IResult<&[u8], Self> where Self: Sized;
    fn serialize(&self) -> Vec<u8>;
}

pub struct ItemCountListPair<T> 
    where
        T: Serializable
{
    pub length: u16,
    pub data: Vec<T>
}

impl<T> Serializable for ItemCountListPair<T> where T: Serializable {
        fn deserialize(_input: &[u8]) -> IResult<&[u8], ItemCountListPair<T>> {
            panic!("Cannot deserialize as ItemCountListPair");
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


    
    