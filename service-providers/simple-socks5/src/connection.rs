use available_reader::available_reader::AvailableReader;
use io::ErrorKind;
use log::error;
use nymsphinx::addressing::clients::Recipient;
use ordered_buffer::OrderedMessageBuffer;
use ordered_buffer::{OrderedMessage, OrderedMessageSender};
use socks5_requests::{ConnectionId, RemoteAddress};
use std::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

/// A TCP connection between the Socks5 service provider, which makes
/// outbound requests on behalf of users, and a remote system. Makes the request,
/// reads any response, and returns the response data to the requesting user through
/// the mixnet.
#[derive(Debug)]
pub(crate) struct Connection {
    id: ConnectionId,
    address: RemoteAddress,
    conn: TcpStream,
    return_address: Recipient,
    request_buffer: OrderedMessageBuffer,
    response_sender: OrderedMessageSender,
}

impl Connection {
    pub(crate) async fn new(
        id: ConnectionId,
        address: RemoteAddress,
        initial_data: &[u8],
        response_sender: OrderedMessageSender,
        request_buffer: OrderedMessageBuffer,
        return_address: Recipient,
    ) -> io::Result<Self> {
        let conn = match TcpStream::connect(&address).await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("error while connecting to {:?} ! - {:?}", address, err);
                return Err(err);
            }
        };
        let mut connection = Connection {
            id,
            address,
            conn,
            request_buffer,
            response_sender,
            return_address,
        };
        // get initial data, if there is any, from the request_buffer
        connection.send_data(&initial_data).await?;
        Ok(connection)
    }

    pub(crate) fn return_address(&self) -> Recipient {
        self.return_address.clone()
    }

    pub(crate) async fn send_data(&mut self, data: &[u8]) -> io::Result<()> {
        let message = match OrderedMessage::try_from_bytes(data.to_vec()) {
            Ok(message) => message,
            _ => {
                return Err(io::Error::new(
                    ErrorKind::Other,
                    "Can't ordered message bytes from request",
                ))
            }
        };
        self.request_buffer.write(message);
        if let Some(data) = self.request_buffer.read() {
            println!("Sending {} bytes to {}", data.len(), self.address);

            if let Err(err) = self.conn.write_all(&data).await {
                error!(
                    "tried to write to (presumably) closed connection - {:?}",
                    err
                );
            }
        }
        Ok(())
    }

    /// Read response data by looping, waiting for anything we get back from the
    /// remote server. Returns once it times out or the connection closes.
    pub(crate) async fn try_read_response_data(&mut self) -> io::Result<OrderedMessage> {
        let available_reader = AvailableReader::new(&mut self.conn);
        let data = available_reader.await?.to_vec();
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Connection closed",
            ));
        }

        let message = self.response_sender.into_message(data);
        Ok(message)
    }
}
