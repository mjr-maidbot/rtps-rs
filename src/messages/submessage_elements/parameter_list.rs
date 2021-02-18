use crate::common::size_tracking_context::SizeTrackingContext;
use crate::messages::submessage_elements::parameter::Parameter;
use crate::structure::parameter_id::ParameterId;
use speedy::{Context, Readable, Reader, Writable, Writer};

/// ParameterList is used as part of several messages to encapsulate
/// QoS parameters that may affect the interpretation of the message.
/// The encapsulation of the parameters follows a mechanism that allows
/// extensions to the QoS without breaking backwards compatibility.
#[derive(Debug, PartialEq)]
pub struct ParameterList {
    parameters: Vec<Parameter>,
}

impl<'a, C: SizeTrackingContext> Readable<'a, C> for ParameterList {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut parameters = Vec::new();

        loop {
            let parameter: Parameter = reader.read_value()?;

            if parameter.get_id() == ParameterId::PID_PAD {
                continue;
            }
            if parameter.get_id() == ParameterId::PID_SENTINEL {
                break;
            }

            parameters.push(parameter);
        }

        Ok(ParameterList {
            parameters,
        })
    }
}

impl<C: Context> Writable<C> for ParameterList {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        let mut need_sentinel = true;

        // Stop early if a sentinel is encountered and drop the remaining
        // parameters.
        for parameter in &self.parameters {
            writer.write_value(parameter)?;
            if parameter.is_sentinel() {
                need_sentinel = false;
                break;
            }
        }

        // Write a sentinel if the list did not already contain one.
        if need_sentinel {
            writer.write_value(&Parameter::new_sentinel())?;
        }

        Ok(())
    }
}
