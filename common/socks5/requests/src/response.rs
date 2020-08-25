use crate::ConnectionId;
use ordered_buffer::MessageError;
use ordered_buffer::OrderedMessage;

#[derive(Debug, PartialEq)]
pub enum ResponseError {
    ConnectionIdTooShort,
    NoData,
    OrderedMessageDeserializationError,
}

impl From<MessageError> for ResponseError {
    fn from(e: MessageError) -> Self {
        match e {
            _ => ResponseError::OrderedMessageDeserializationError,
        }
    }
}
/// A remote network response retrieved by the Socks5 service provider. This
/// can be serialized and sent back through the mixnet to the requesting
/// application.
#[derive(Debug)]
pub struct Response {
    pub message: OrderedMessage, // change to ordered message
    pub connection_id: ConnectionId,
}

impl Response {
    /// Constructor for responses
    pub fn new(connection_id: ConnectionId, message: OrderedMessage) -> Self {
        Response {
            connection_id,
            message,
        }
    }

    pub fn try_from_bytes(b: &[u8]) -> Result<Response, ResponseError> {
        if b.is_empty() {
            return Err(ResponseError::NoData);
        }

        if b.len() < 8 {
            return Err(ResponseError::ConnectionIdTooShort);
        }

        let mut connection_id_bytes = b.to_vec();
        let data = connection_id_bytes.split_off(8);

        let connection_id = u64::from_be_bytes([
            connection_id_bytes[0],
            connection_id_bytes[1],
            connection_id_bytes[2],
            connection_id_bytes[3],
            connection_id_bytes[4],
            connection_id_bytes[5],
            connection_id_bytes[6],
            connection_id_bytes[7],
        ]);

        let message = OrderedMessage::try_from_bytes(data)?;
        let response = Response::new(connection_id, message);
        Ok(response)
    }

    /// Serializes the response into bytes so that it can be sent back through
    /// the mixnet to the requesting application. The format is
    /// | 8 connection_id_bytes | 8 message index bytes | data |
    pub fn into_bytes(self) -> Vec<u8> {
        self.connection_id
            .to_be_bytes()
            .iter()
            .cloned()
            .chain(self.message.into_bytes())
            .collect()
    }
}

#[cfg(test)]
mod serializing_socks5_responses_into_bytes {
    use super::*;

    #[test]
    fn works() {
        let message = OrderedMessage {
            data: vec![222],
            index: 1,
        };
        let response = Response::new(111, message);
        let bytes = response.into_bytes();

        assert_eq!(
            vec![0, 0, 0, 0, 0, 0, 0, 111, 0, 0, 0, 0, 0, 0, 0, 1, 222],
            bytes
        );
    }
}

#[cfg(test)]
mod constructing_socks5_responses_from_bytes {
    use super::*;

    #[test]
    fn fails_when_zero_bytes_are_supplied() {
        let response_bytes = Vec::new();

        assert_eq!(
            ResponseError::NoData,
            Response::try_from_bytes(&response_bytes).unwrap_err()
        );
    }

    #[test]
    fn fails_when_connection_id_bytes_are_too_short() {
        let response_bytes = vec![0, 1, 2, 3, 4, 5, 6];
        assert_eq!(
            ResponseError::ConnectionIdTooShort,
            Response::try_from_bytes(&response_bytes).unwrap_err()
        );
    }

    #[test]
    fn works_when_there_is_no_data() {
        let message = OrderedMessage {
            data: Vec::new(),
            index: 1,
        };
        let mut message_bytes = message.clone().into_bytes();
        let mut response_bytes: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        response_bytes.append(&mut message_bytes);
        let expected = Response::new(u64::from_be_bytes([0, 1, 2, 3, 4, 5, 6, 7]), message);
        let actual = Response::try_from_bytes(&response_bytes).unwrap();
        assert_eq!(expected.connection_id, actual.connection_id);
        assert_eq!(expected.message, actual.message);
    }

    #[test]
    fn works_when_there_is_data() {
        let response_bytes = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0, 0, 0, 0, 1, 255, 255, 255,
        ];
        let expected = Response::new(
            u64::from_be_bytes([0, 1, 2, 3, 4, 5, 6, 7]),
            OrderedMessage {
                index: 1,
                data: vec![255, 255, 255],
            },
        );
        let actual = Response::try_from_bytes(&response_bytes).unwrap();
        assert_eq!(expected.connection_id, actual.connection_id);
        assert_eq!(expected.message, actual.message);
    }
}
