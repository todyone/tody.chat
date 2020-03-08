use anyhow::Error;
use bytes::{Buf, BufMut, BytesMut};
use futures::stream::{Fuse, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

pub type NetworkConnection<T> = Fuse<Framed<TcpStream, T>>;

pub fn wrap<T, I>(stream: TcpStream) -> NetworkConnection<T>
where
    T: Encoder<I> + Decoder + Default,
{
    Framed::new(stream, T::default()).fuse()
}

/// Type of message's size field.
type Size = u32;

/// Size of message's size field.
const SIZE: usize = mem::size_of::<Size>();

// TODO: Reduce to 1Mb and have a separate transfer activities
// and cache for all transfers with limits.
/// Maximal size of a message. Equals **10Mb**.
const MAX_SIZE: usize = 1024 * 1024 * 10;

#[derive(Debug)]
pub struct ProtocolCodec<E, D> {
    size: usize,
    _encode: PhantomData<E>,
    _decode: PhantomData<D>,
}

impl<E, D> Default for ProtocolCodec<E, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, D> ProtocolCodec<E, D> {
    /// Creates new instance of the codec.
    pub fn new() -> Self {
        Self {
            size: 0,
            _encode: PhantomData,
            _decode: PhantomData,
        }
    }
}

#[derive(Error, Debug)]
enum CodecError {
    #[error("Message too big: {size}. Max: {max}.")]
    MessageTooBig { size: usize, max: usize },
}

impl<E, D> Encoder<E> for ProtocolCodec<E, D>
where
    E: Serialize,
{
    type Error = Error;

    fn encode(&mut self, item: E, bytes: &mut BytesMut) -> Result<(), Self::Error> {
        let data = serde_cbor::to_vec(&item)?;
        let size = data.len().try_into()?;
        bytes.put_u32(size);
        bytes.put(data.as_ref());
        Ok(())
    }
}

impl<E, D> Decoder for ProtocolCodec<E, D>
where
    D: DeserializeOwned,
{
    type Item = D;
    type Error = Error;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.size == 0 {
            if bytes.len() >= SIZE {
                self.size = bytes.get_u32().try_into()?;
            }
            if self.size > MAX_SIZE {
                let err = CodecError::MessageTooBig {
                    size: self.size,
                    max: MAX_SIZE,
                };
                return Err(err.into());
            }
        }
        if self.size > 0 && self.size <= bytes.len() {
            let data = bytes.split_to(self.size);
            self.size = 0;
            let msg = serde_cbor::from_slice(data.as_ref())?;
            Ok(Some(msg))
        } else {
            Ok(None)
        }
    }
}
