use std::marker::PhantomData;
use std::pin::Pin;

use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};
use tokio_serde::{Deserializer, Serializer};
use tonic::Status;

#[derive(Default, Clone, Copy)]
pub struct Encoder<T, C> {
    _pd_payload: PhantomData<T>,
    _pd_codec: PhantomData<C>,
}

impl<T, C> tonic::codec::Encoder for Encoder<T, C>
where
    T: Serialize + Unpin,
    C: Serializer<T> + std::default::Default + Unpin,
    C::Error: std::fmt::Display,
{
    type Item = T;
    type Error = tonic::Status;
    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        let mut serializer: C = C::default();
        let bytes = Pin::new(&mut serializer)
            .serialize(&item)
            .map_err(|serde_err| {
                Status::internal(format!("Error in serde deserialize {}", serde_err))
            })?;
        Ok(dst.put(bytes))
    }
}

#[derive(Default, Clone, Copy)]
pub struct Decoder<T, C> {
    _pd_payload: PhantomData<T>,
    _pd_codec: PhantomData<C>,
}

impl<T, C> tonic::codec::Decoder for Decoder<T, C>
where
    T: for<'a> Deserialize<'a> + Unpin,
    C: Deserializer<T> + std::default::Default + Unpin,
    C::Error: std::fmt::Display,
{
    type Item = T;
    type Error = tonic::Status;
    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut deserializer = C::default();
        let mut bytes = bytes::BytesMut::new();
        bytes.extend_from_slice(&src.to_bytes());
        let result = Pin::new(&mut deserializer)
            .deserialize(&bytes)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))?;
        Ok(Some(result))
    }
}

pub struct Codec<T, U, E, D> {
    _pd: PhantomData<(T, U, E, D)>,
}

impl<T, U, E, D> Default for Codec<T, U, E, D> {
    fn default() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U, E, D> tonic::codec::Codec for Codec<T, U, E, D>
where
    T: Send + Sync + Serialize + Unpin + 'static,
    U: Send + Sync + for<'a> Deserialize<'a> + Unpin + 'static,
    E: Serializer<T> + std::default::Default + Unpin + Send + Sync + 'static,
    E::Error: std::fmt::Display,
    D: Deserializer<U> + std::default::Default + Unpin + Send + Sync + 'static,
    D::Error: std::fmt::Display,
{
    type Encode = T;
    type Decode = U;
    type Encoder = Encoder<T, E>;
    type Decoder = Decoder<U, D>;
    fn encoder(&mut self) -> Encoder<T, E> {
        Encoder {
            _pd_payload: PhantomData,
            _pd_codec: PhantomData,
        }
    }
    fn decoder(&mut self) -> Decoder<U, D> {
        Decoder {
            _pd_payload: PhantomData,
            _pd_codec: PhantomData,
        }
    }
}

pub type JsonCodec<T, U> =
    Codec<T, U, tokio_serde::formats::Json<T, T>, tokio_serde::formats::Json<U, U>>;
pub type BincodeCodec<T, U> =
    Codec<T, U, tokio_serde::formats::Bincode<T, T>, tokio_serde::formats::Bincode<U, U>>;
pub type CborCodec<T, U> =
    Codec<T, U, tokio_serde::formats::Cbor<T, T>, tokio_serde::formats::Cbor<U, U>>;
pub type MessagePackCodec<T, U> =
    Codec<T, U, tokio_serde::formats::MessagePack<T, T>, tokio_serde::formats::MessagePack<U, U>>;
