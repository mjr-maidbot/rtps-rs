use crate::messages::submessage_elements::representation_identifier::RepresentationIdentifier;
use speedy::{Context, Readable, Reader, Writable, Writer};

/// All SerializedPayload shall start with the SerializedPayloadHeader defined
/// below. The header provides information about the representation of the data
/// that follows.
///
/// TheRepresentationIdentifier is used to identify the data representation
/// used. The RepresentationOptions shall be interpreted in the context of the
/// RepresentationIdentifier, such that each RepresentationIdentifier may define
/// the representation_options that it requires.
#[derive(Debug, PartialEq)]
pub struct SerializedPayloadHeader {
    pub representation_identifier: RepresentationIdentifier,
    pub representation_options: [u8;2],
}

impl Default for SerializedPayloadHeader {
    fn default() -> SerializedPayloadHeader {
        SerializedPayloadHeader {
            representation_identifier: RepresentationIdentifier::default(),
            representation_options: [0;2],
        }
    }
}

impl<'a, C: Context> Readable<'a, C> for SerializedPayloadHeader {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let representation_identifier: RepresentationIdentifier = reader.read_value()?;
        let representation_options = [reader.read_u8()?, reader.read_u8()?];

        Ok(SerializedPayloadHeader {
            representation_identifier,
            representation_options,
        })
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        4
    }
}

impl<C: Context> Writable<C> for SerializedPayloadHeader {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.representation_identifier)?;
        writer.write_slice(&self.representation_options)?;

        Ok(())
    }
}
