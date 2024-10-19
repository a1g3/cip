use nom::number::complete::le_u16;
use strum_macros::EnumIter;

use crate::common::Serializable;

pub struct MessageRouter {
    pub objects: Vec<u16>
}

impl Serializable for MessageRouter {
    fn deserialize(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized {
        let (input, number_objects) = le_u16(input)?;
        let mut objects = Vec::new();

        let mut remaining_input = input;
        println!("Number of objects is {}", number_objects);
        if number_objects > 0 {
            for _ in 0..number_objects {
                let (input, object_num) = le_u16(remaining_input)?;
                remaining_input = input;
                objects.push(object_num)
            }
        }
        objects.sort();
        return Ok((remaining_input, MessageRouter { objects }))
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(EnumIter)]
pub enum MessageRouterResponseStatusCodes {
    Success = 0x00,
    ConnectionProblem = 0x01,
    ResourceUnavailable = 0x02,
    InvalidParameterValue = 0x03,
    PathSegmentError = 0x04,
    PathDestinationUnknown = 0x05,
    PartialTranser = 0x06,
    ConnectionLost = 0x07,
    ServiceNotSupported = 0x08,
    InvalidAttributeValue = 0x09,
    AttributeListError = 0x0A,
    AlreadyInState = 0x0B,
    ObjectStateConflict = 0x0C,
    ObjectAlreadyExists = 0x0D,
    AttributeNotSettable = 0x0E,
    PrivilegeViolation = 0x0F,
    DeviceStateConflict = 0x10,
    ReplyToLarge = 0x11,
    FragmentationOfPrimitive = 0x12,
    NotEnoughData = 0x13,
    AttributeNotSupported = 0x14,
    TooMuchData = 0x15,
    ObjectDoesNotExist = 0x16,
    ServiceFragmentationOutOfSequence = 0x17,
    NoStoreAttribute = 0x18,
    StorageOperationFailure = 0x19,
    RequestToLarge = 0x1A,
    ResponeToLarge = 0x1B,
    MissingAttributeList = 0x1C,
    InvalidAttibuteList = 0x1D,
    EmbeddedServiceError = 0x1E,
    VendorSpecifiedError = 0x1F,
    InvalidParameter = 0x20,
    WriteOnceValue = 0x21,
    InvalidReply = 0x22,
    BufferOverflow = 0x23,
    MessageFormatError = 0x24,
    KeyFailure = 0x25,
    PathSizeInvalid = 0x26,
    UnexpectedAttribute = 0x27,
    InvalidMemberId = 0x28,
    MemberNotSettable = 0x29,
    GroupTwoOnly = 0x2A,
    ModbusError = 0x2B,
    AttributeNotGetable = 0x2C,
    InstanceNotDeletable = 0x2D,
    ServiceNotSupportedForPath = 0x2E
}