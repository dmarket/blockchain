
#[macro_export]
macro_rules! evo_message {
    (
    $(#[$attr:meta])*
    struct $name:ident {
        const TYPE = $extension:expr;
        const ID = $id:expr;

        $(
        $(#[$field_attr:meta])*
        $field_name:ident : $field_type:ty
        ),*
        $(,)*
    }) => (
        #[derive(Clone, PartialEq)]
        $(#[$attr])*
        pub struct $name {
            raw: $crate::messages::RawMessage
        }

        impl $crate::messages::Message for $name {
            fn from_raw(raw: $crate::messages::RawMessage)
                -> Result<$name, $crate::encoding::Error> {
                let min_message_size = $name::__ex_header_size() as usize
                            + $crate::messages::HEADER_LENGTH as usize
                            + $crate::crypto::SIGNATURE_LENGTH as usize;
                if raw.len() < min_message_size {
                    return Err($crate::encoding::Error::UnexpectedlyShortPayload {
                        actual_size: raw.len() as $crate::encoding::Offset,
                        minimum_size: min_message_size as $crate::encoding::Offset,
                    });
                }

                // Check identifiers
                if raw.version() != $crate::messages::PROTOCOL_MAJOR_VERSION {
                    return Err($crate::encoding::Error::UnsupportedProtocolVersion {
                        version: $crate::messages::PROTOCOL_MAJOR_VERSION
                    });
                }
                if raw.network_id() != $crate::messages::TEST_NETWORK_ID {
                    return Err($crate::encoding::Error::IncorrectNetworkId {
                        network_id: $crate::messages::TEST_NETWORK_ID
                    });
                }
                if raw.message_type() != $id {
                    return Err($crate::encoding::Error::IncorrectMessageType {
                        message_type: $id
                    });
                }
                if raw.service_id() != $extension {
                    return Err($crate::encoding::Error::IncorrectServiceId {
                        service_id: $extension
                    });
                }

                // Check body
                let body_len = <Self>::check_fields(&raw)?;
                if body_len.unchecked_offset() as usize +
                    $crate::crypto::SIGNATURE_LENGTH as usize != raw.len()  {
                   return Err("Incorrect raw message length.".into())
                }

                Ok($name { raw: raw })
            }


            fn raw(&self) -> &$crate::messages::RawMessage {
                &self.raw
            }
        }

        impl<'a> $crate::encoding::SegmentField<'a> for $name {

            fn item_size() -> $crate::encoding::Offset {
                1
            }

            fn count(&self) -> $crate::encoding::Offset {
                self.raw.len() as $crate::encoding::Offset
            }

            fn extend_buffer(&self, buffer: &mut Vec<u8>) {
                buffer.extend_from_slice(self.raw.as_ref().as_ref())
            }

            unsafe fn from_buffer(buffer: &'a [u8],
                                    from: $crate::encoding::Offset,
                                    count: $crate::encoding::Offset) -> Self {
                let raw_message: $crate::messages::RawMessage =
                                    $crate::encoding::SegmentField::from_buffer(buffer,
                                                                from,
                                                                count);
                $crate::messages::Message::from_raw(raw_message).unwrap()
            }

            fn check_data(buffer: &'a [u8],
                    from: $crate::encoding::CheckedOffset,
                    count: $crate::encoding::CheckedOffset,
                    latest_segment: $crate::encoding::CheckedOffset)
              -> $crate::encoding::Result {
                let latest_segment_origin = <$crate::messages::RawMessage as
                                $crate::encoding::SegmentField>::check_data(buffer,
                                                                from,
                                                                count,
                                                                latest_segment)?;
                // TODO: remove this allocation,
                // by allowing creating message from borrowed data (ECR-156)
                let raw_message: $crate::messages::RawMessage =
                                    unsafe { $crate::encoding::SegmentField::from_buffer(buffer,
                                                                from.unchecked_offset(),
                                                                count.unchecked_offset())};
                let _: $name = $crate::messages::Message::from_raw(raw_message)?;
                Ok(latest_segment_origin)
            }
        }

        impl $name {
            #[cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
            /// Creates message and signs it.
            #[allow(unused_mut)]
            pub fn new($($field_name: $field_type,)*) -> $name {
                use $crate::messages::{RawMessage, MessageWriter};
                let mut writer = MessageWriter::new(
                    $crate::messages::PROTOCOL_MAJOR_VERSION,
                    $crate::messages::TEST_NETWORK_ID,
                    $extension, $id, $name::__ex_header_size() as usize,
                );
                __ex_for_each_field!(
                    __ex_message_write_field, (writer),
                    $( ($(#[$field_attr])*, $field_name, $field_type) )*
                );
                $name { raw: RawMessage::new(writer.to_message_buffer()) }
            }

            /// Creates message and appends existing signature.
            #[cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
            #[allow(dead_code, unused_mut)]
            pub fn new_with_signature($($field_name: $field_type,)*
                                      signature: &$crate::crypto::Signature) -> $name {
                use $crate::messages::{RawMessage, MessageWriter};
                let mut writer = MessageWriter::new(
                    $crate::messages::PROTOCOL_MAJOR_VERSION,
                    $crate::messages::TEST_NETWORK_ID,
                    $extension, $id, $name::__ex_header_size() as usize,
                );
                __ex_for_each_field!(
                    __ex_message_write_field, (writer),
                    $( ($(#[$field_attr])*, $field_name, $field_type) )*
                );
                $name { raw: RawMessage::new(writer.append_signature(signature)) }
            }

            #[allow(unused_variables)]
            fn check_fields(raw_message: &$crate::messages::RawMessage)
            -> $crate::encoding::Result {
                let header_length =
                    $crate::messages::HEADER_LENGTH as $crate::encoding::Offset;
                let latest_segment = ($name::__ex_header_size() + header_length).into();
                __ex_for_each_field!(
                    __ex_message_check_field, (latest_segment, raw_message),
                    $( ($(#[$field_attr])*, $field_name, $field_type) )*
                );
                Ok(latest_segment)
            }

            /// Returns `message_id` usable for matching.
            #[allow(dead_code)]
            pub fn message_id() -> u16 {
                $id
            }

            /// Returns `service_id` usable for matching.
            #[allow(dead_code)]
            pub fn service_id() -> u16 {
                $extension
            }

            __ex_for_each_field!(
                __ex_message_mk_field, (),
                $( ($(#[$field_attr])*, $field_name, $field_type) )*
            );

            #[doc(hidden)]
            fn __ex_header_size() -> $crate::encoding::Offset {
                __ex_header_size!($($field_type),*)
            }
        }

        impl AsRef<$crate::messages::RawMessage> for $name {
            fn as_ref(&self) -> &$crate::messages::RawMessage {
                $crate::messages::Message::raw(self)
            }
        }

        impl $crate::storage::StorageValue for $name {

            fn into_bytes(self) -> Vec<u8> {
                self.raw.as_ref().as_ref().to_vec()
            }

            fn from_bytes(value: ::std::borrow::Cow<[u8]>) -> Self {
                $name {
                    raw: $crate::messages::RawMessage::new(
                        $crate::messages::MessageBuffer::from_vec(value.into_owned()))
                }
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
                -> Result<(), ::std::fmt::Error> {
                fmt.debug_struct(stringify!($name))
                 $(.field(stringify!($field_name), &self.$field_name()))*
                   .finish()
            }
        }

    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ex_message_mk_field {
    (
        (),
        $(#[$field_attr:meta])*, $field_name:ident, $field_type:ty, $from:expr, $to:expr
    ) => {
        $(#[$field_attr])*
        pub fn $field_name(&self) -> $field_type {
            unsafe { self.raw.read::<$field_type>($from, $to) }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ex_message_write_field {
    (
        ($writer:ident),
        $(#[$field_attr:meta])*,
        $field_name:ident,
        $field_type:ty,
        $from:expr,
        $to:expr
    ) => {
        $writer.write($field_name, $from, $to);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ex_message_check_field {
    (
        ($latest_segment:ident, $raw_message:ident),
        $(#[$field_attr:meta])*,
        $field_name:ident,
        $field_type:ty,
        $from:expr,
        $to:expr
    ) => {
        let $latest_segment =
            $raw_message.check::<$field_type>($from.into(), $to.into(), $latest_segment)?;
    };
}
