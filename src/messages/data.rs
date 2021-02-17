use crate::messages::data_submessage_flags::DataSubmessageFlags;
use crate::messages::submessage_elements::parameter_list::ParameterList;
use crate::messages::submessage_elements::serialized_payload::SerializedPayload;
use crate::messages::submessage_flag::SubmessageFlag;
use crate::structure::entity_id::EntityId_t;
use crate::structure::sequence_number::SequenceNumber_t;
use crate::structure::size_tracking_context::SizeTrackingContext;
use speedy::{Context, Endianness, Readable, Reader, Writable, Writer};

/// This is a speedy::Context for processing Data submessages. It contains flags
/// that are used in message processing, and it also implements the
/// SizeTrackingContext trait in order to track the length of the input buffer
/// during deserialization.
pub struct DataContext {
    flags: DataSubmessageFlags,
    length_remaining: usize,
}

impl DataContext {
    pub fn new(flags: SubmessageFlag, length_remaining: usize) -> DataContext {
        DataContext {
            flags: flags.into(),
            length_remaining,
        }
    }
}

impl Context for DataContext {
    type Error = speedy::Error;

    fn endianness(&self) -> Endianness {
        self.flags.endianness()
    }
}

impl SizeTrackingContext for DataContext {
    fn subtract_from_remaining(&mut self, length: usize) {
        self.length_remaining -= length;
    }

    fn length_remaining(&self) -> usize {
        self.length_remaining
    }
}

/// This Submessage is sent from an RTPS Writer (NO_KEY or WITH_KEY)
/// to an RTPS Reader (NO_KEY or WITH_KEY)
///
/// The Submessage notifies the RTPS Reader of a change to
/// a data-object belonging to the RTPS Writer. The possible changes
/// include both changes in value as well as changes to the lifecycle
/// of the data-object.
#[derive(Debug, PartialEq)]
pub struct Data {
    /// Identifies the RTPS Reader entity that is being informed of the change
    /// to the data-object.
    pub reader_id: EntityId_t,

    /// Identifies the RTPS Writer entity that made the change to the
    /// data-object.
    pub writer_id: EntityId_t,

    /// Uniquely identifies the change and the relative order for all changes
    /// made by the RTPS Writer identified by the writerGuid. Each change
    /// gets a consecutive sequence number. Each RTPS Writer maintains is
    /// own sequence number.
    pub writer_sn: SequenceNumber_t,

    /// Contains QoS that may affect the interpretation of the message.
    /// Present only if the InlineQosFlag is set in the header.
    pub inline_qos: Option<ParameterList>,

    /// If the DataFlag is set, then it contains the encapsulation of
    /// the new value of the data-object after the change.
    /// If the KeyFlag is set, then it contains the encapsulation of
    /// the key of the data-object the message refers to.
    /// If the NonStandardPayloadFlag is set, then it contains data
    /// that is "not formatted according to section 10".
    pub serialized_payload: Option<SerializedPayload>,
}

impl<'a> Readable<'a, DataContext> for Data {
    #[inline]
    fn read_from<R: Reader<'a, DataContext>>(
        reader: &mut R
    ) -> Result<Self, <DataContext as Context>::Error>
    {
        let flags = reader.context().flags;

        // skip over "extra flags" and "octets to inline qos"
        reader.skip_bytes(4)?;
        reader.context_mut().subtract_from_remaining(4);

        let reader_id: EntityId_t = reader.read_value()?;
        reader.context_mut().subtract_from_remaining(
            <EntityId_t as Readable<DataContext>>::minimum_bytes_needed()
        );

        let writer_id: EntityId_t = reader.read_value()?;
        reader.context_mut().subtract_from_remaining(
            <EntityId_t as Readable<DataContext>>::minimum_bytes_needed()
        );

        let writer_sn: SequenceNumber_t = reader.read_value()?;
        reader.context_mut().subtract_from_remaining(
            <SequenceNumber_t as Readable<DataContext>>::minimum_bytes_needed()
        );

        let inline_qos: Option<ParameterList> =
            match flags.inline_qos() {
                true => {
                    let parameter_list: ParameterList = reader.read_value()?;
                    Some(parameter_list)
                },
                false => None,
            };

        let serialized_payload: Option<SerializedPayload> =
            match flags.any_payload() {
                true => {
                    let serialized_payload: SerializedPayload = reader.read_value()?;
                    Some(serialized_payload)
                },
                false => None,
            };

        Ok(Data {
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        })
    }
}

impl<C: Context> Writable<C> for Data {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // From spec document section 9.4.5.3.2: "This version of the protocol
        // should set all the bits in the extraFlags to zero".
        writer.write_u8(0)?;
        writer.write_u8(0)?;

        // Write "octets to inline QoS", which will always be 16 bytes.
        writer.write_u16(16)?;

        writer.write_value(&self.reader_id)?;
        writer.write_value(&self.writer_id)?;
        writer.write_value(&self.writer_sn)?;

        if let Some(ref inline_qos) = self.inline_qos {
            writer.write_value(inline_qos)?;
        }

        if let Some(ref serialized_payload) = self.serialized_payload {
            writer.write_value(serialized_payload)?;
        }

        Ok(())
    }
}
