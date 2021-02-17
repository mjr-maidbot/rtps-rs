use crate::messages::submessage_elements::parameter_list::ParameterList;
use crate::messages::submessage_elements::serialized_payload_header::SerializedPayloadHeader;
use crate::structure::size_tracking_context::SizeTrackingContext;
use speedy::{Context, Readable, Reader, Writable, Writer};

/// A SerializedPayload is either a ParameterList or user-defined data in an
/// unspecified format.
#[derive(Debug, PartialEq)]
pub enum SerializedPayloadContent {
    ParameterList(ParameterList),
    UserDefined(Box<[u8]>),
}

/// A SerializedPayload contains the serialized representation of
/// either value of an application-defined data-object or
/// the value of the key that uniquely identifies the data-object
#[derive(Debug, PartialEq)]
pub struct SerializedPayload {
    pub header: SerializedPayloadHeader,
    pub content: SerializedPayloadContent,
}

impl<'a, C: SizeTrackingContext> Readable<'a, C> for SerializedPayload {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let header: SerializedPayloadHeader = reader.read_value()?;
        reader.context_mut().subtract_from_remaining(
            <SerializedPayloadHeader as Readable<C>>::minimum_bytes_needed()
        );

        let representation_identifier = header.representation_identifier;
        let content = if representation_identifier.is_parameter_list() {
            // The contents of the SerializedPayload are to be parsed as a
            // ParameterList.
            let parameter_list: ParameterList = reader.read_value()?;
            SerializedPayloadContent::ParameterList(parameter_list)
        } else {
            // The contents of the SerializedPayload are to be parsed as user-
            // defined data in an unspecified format.
            let mut payload = vec![0; reader.context().length_remaining()].into_boxed_slice();
            reader.read_bytes(&mut payload)?;
            SerializedPayloadContent::UserDefined(payload)
        };

        Ok(SerializedPayload{
            header,
            content,
        })
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        4
    }
}

impl<C: Context> Writable<C> for SerializedPayload {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.header)?;
        match self.content {
            SerializedPayloadContent::ParameterList(ref parameter_list) => {
                // The RepresentationIdentifier from the SubmessageHeader
                // indicates the endianness to be used to write the parameter
                // list.
                let bytes =
                    parameter_list.write_to_vec_with_ctx(self.header.representation_identifier)?;
                writer.write_bytes(&bytes)?;
            },
            SerializedPayloadContent::UserDefined(ref user_defined) => {
                writer.write_bytes(user_defined)?;
            },
        }

        Ok(())
    }
}
