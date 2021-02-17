use crate::messages::submessage_flag::SubmessageFlag;
use speedy::{Context, Endianness};

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct DataSubmessageFlags {
    submessage_flag: SubmessageFlag,
}

impl DataSubmessageFlags {
    pub const INLINE_QOS_FLAG_MASK: u8 = 0x02;
    pub const DATA_FLAG_MASK: u8 = 0x04;
    pub const KEY_FLAG_MASK: u8 = 0x08;
    pub const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x10;

    pub fn inline_qos(&self) -> bool {
        self.is_flag_set(Self::INLINE_QOS_FLAG_MASK)
    }

    pub fn data_payload(&self) -> bool {
        self.is_flag_set(Self::DATA_FLAG_MASK) && !self.is_flag_set(Self::KEY_FLAG_MASK)
    }

    pub fn key_payload(&self) -> bool {
        !self.is_flag_set(Self::DATA_FLAG_MASK) && self.is_flag_set(Self::KEY_FLAG_MASK)
    }

    pub fn non_standard_payload(&self) -> bool {
        self.is_flag_set(Self::NON_STANDARD_PAYLOAD_FLAG_MASK)
    }

    pub fn any_payload(&self) -> bool {
        self.data_payload() || self.key_payload() || self.non_standard_payload()
    }

    #[inline]
    pub fn is_flag_set(&self, mask: u8) -> bool {
        self.submessage_flag.flags & mask != 0
    }
}

impl Context for DataSubmessageFlags {
    type Error = speedy::Error;

    fn endianness(&self) -> Endianness {
        self.submessage_flag.endianness()
    }
}

impl From<SubmessageFlag> for DataSubmessageFlags {
    fn from(submessage_flag: SubmessageFlag) -> DataSubmessageFlags {
        DataSubmessageFlags {
            submessage_flag
        }
    }
}
