use speedy::{Context, Endianness, Readable, Reader, Writable, Writer};
use std::convert::TryFrom;

/// The RepresentationIdentifier is used to identify the data representation
/// used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RepresentationIdentifier {
    CDR_BE,
    CDR_LE,
    PL_CDR_BE,
    PL_CDR_LE,
    CDR2_BE,
    CDR2_LE,
    PL_CDR2_BE,
    PL_CDR2_LE,
    D_CDR_BE,
    D_CDR_LE,
    XML,
}

impl RepresentationIdentifier {
    pub const VALUE_CDR_BE: [u8; 2] = [0x00, 0x00];
    pub const VALUE_CDR_LE: [u8; 2] = [0x00, 0x01];
    pub const VALUE_PL_CDR_BE: [u8; 2] = [0x00, 0x02];
    pub const VALUE_PL_CDR_LE: [u8; 2] = [0x00, 0x03];
    pub const VALUE_CDR2_BE: [u8; 2] = [0x00, 0x10];
    pub const VALUE_CDR2_LE: [u8; 2] = [0x00, 0x11];
    pub const VALUE_PL_CDR2_BE: [u8; 2] = [0x00, 0x12];
    pub const VALUE_PL_CDR2_LE: [u8; 2] = [0x00, 0x13];
    pub const VALUE_D_CDR_BE: [u8; 2] = [0x00, 0x14];
    pub const VALUE_D_CDR_LE: [u8; 2] = [0x00, 0x15];
    pub const VALUE_XML: [u8; 2] = [0x00, 0x04];

    pub fn is_parameter_list(&self) -> bool {
        match self {
            RepresentationIdentifier::PL_CDR_BE
          | RepresentationIdentifier::PL_CDR_LE
          | RepresentationIdentifier::PL_CDR2_BE
          | RepresentationIdentifier::PL_CDR2_LE
              => true,
            _ => false,
        }
    }
}

impl Default for RepresentationIdentifier {
    fn default() -> RepresentationIdentifier {
        // This is just a guess; it is not in the spec.
        RepresentationIdentifier::CDR_LE
    }
}

impl Context for RepresentationIdentifier {
    type Error = speedy::Error;

    fn endianness(&self) -> Endianness {
        match self {
            RepresentationIdentifier::CDR_BE
          | RepresentationIdentifier::PL_CDR_BE
          | RepresentationIdentifier::CDR2_BE
          | RepresentationIdentifier::PL_CDR2_BE
          | RepresentationIdentifier::D_CDR_BE
              => Endianness::BigEndian,

            RepresentationIdentifier::CDR_LE
          | RepresentationIdentifier::PL_CDR_LE
          | RepresentationIdentifier::CDR2_LE
          | RepresentationIdentifier::PL_CDR2_LE
          | RepresentationIdentifier::D_CDR_LE
              => Endianness::LittleEndian,

            _ => Endianness::NATIVE,
        }
    }
}

impl TryFrom<[u8; 2]> for RepresentationIdentifier {
    type Error = ();

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        match value {
            RepresentationIdentifier::VALUE_CDR_BE => Ok(RepresentationIdentifier::CDR_BE),
            RepresentationIdentifier::VALUE_CDR_LE => Ok(RepresentationIdentifier::CDR_LE),
            RepresentationIdentifier::VALUE_PL_CDR_BE => Ok(RepresentationIdentifier::PL_CDR_BE),
            RepresentationIdentifier::VALUE_PL_CDR_LE => Ok(RepresentationIdentifier::PL_CDR_LE),
            RepresentationIdentifier::VALUE_CDR2_BE => Ok(RepresentationIdentifier::CDR2_BE),
            RepresentationIdentifier::VALUE_CDR2_LE => Ok(RepresentationIdentifier::CDR2_LE),
            RepresentationIdentifier::VALUE_PL_CDR2_BE => Ok(RepresentationIdentifier::PL_CDR2_BE),
            RepresentationIdentifier::VALUE_PL_CDR2_LE => Ok(RepresentationIdentifier::PL_CDR2_LE),
            RepresentationIdentifier::VALUE_D_CDR_BE => Ok(RepresentationIdentifier::D_CDR_BE),
            RepresentationIdentifier::VALUE_D_CDR_LE => Ok(RepresentationIdentifier::D_CDR_LE),
            RepresentationIdentifier::VALUE_XML => Ok(RepresentationIdentifier::XML),

            _ => Err(()),
        }
    }
}

impl From<RepresentationIdentifier> for [u8; 2] {
    fn from(identifier: RepresentationIdentifier) -> Self {
        match identifier {
            RepresentationIdentifier::CDR_BE => RepresentationIdentifier::VALUE_CDR_BE,
            RepresentationIdentifier::CDR_LE => RepresentationIdentifier::VALUE_CDR_LE,
            RepresentationIdentifier::PL_CDR_BE => RepresentationIdentifier::VALUE_PL_CDR_BE,
            RepresentationIdentifier::PL_CDR_LE => RepresentationIdentifier::VALUE_PL_CDR_LE,
            RepresentationIdentifier::CDR2_BE => RepresentationIdentifier::VALUE_CDR2_BE,
            RepresentationIdentifier::CDR2_LE => RepresentationIdentifier::VALUE_CDR2_LE,
            RepresentationIdentifier::PL_CDR2_BE => RepresentationIdentifier::VALUE_PL_CDR2_BE,
            RepresentationIdentifier::PL_CDR2_LE => RepresentationIdentifier::VALUE_PL_CDR2_LE,
            RepresentationIdentifier::D_CDR_BE => RepresentationIdentifier::VALUE_D_CDR_BE,
            RepresentationIdentifier::D_CDR_LE => RepresentationIdentifier::VALUE_D_CDR_LE,
            RepresentationIdentifier::XML => RepresentationIdentifier::VALUE_XML,
        }
    }
}

impl<'a, C: Context> Readable<'a, C> for RepresentationIdentifier {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let value = [reader.read_u8()?, reader.read_u8()?];
        let identifier = RepresentationIdentifier::try_from(value)
            .map_err(|_| speedy::Error::custom("illegal representation identifier"))?;

        Ok(identifier)
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        2
    }
}

impl<C: Context> Writable<C> for RepresentationIdentifier {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_slice(&<[u8; 2]>::from(*self))?;

        Ok(())
    }
}
