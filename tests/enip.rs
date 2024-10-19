#[cfg(test)]
mod tests {
    use std::io;
    use nom::lib::std::result::Result::Err;

    use cip::{common::Serializable, enip::SendUnitData};


    #[test]
    fn sendunitdata_deserialize_invalid() {
        let input: [u8; 1] = [0];
        let result = SendUnitData::deserialize(&input);
        assert!(result.is_err());
    }

    #[test]
    fn sendunitdata_deserialize_valid() {
        let input: [u8] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = SendUnitData::deserialize(&input);
        assert!(result.is_err());
    }
}