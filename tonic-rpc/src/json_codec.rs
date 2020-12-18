use std::marker::PhantomData;
use std::pin::Pin;

use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};
use tokio_serde::{Deserializer, Serializer};
use tonic::Status;

#[derive(Default, Clone, Copy)]
pub struct Encoder<T> {
    _pd: PhantomData<T>,
}

impl<T> tonic::codec::Encoder for Encoder<T>
where
    T: Serialize + Unpin,
{
    type Item = T;
    type Error = tonic::Status;
    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        let mut serializer: tokio_serde::formats::Json<T, T> =
            tokio_serde::formats::Json::default();
        let bytes = Pin::new(&mut serializer)
            .serialize(&item)
            .map_err(|serde_err| {
                Status::internal(format!("Error in serde deserialize {}", serde_err))
            })?;
        Ok(dst.put(bytes))
    }
}

#[derive(Default, Clone, Copy)]
pub struct Decoder<T> {
    _pd: PhantomData<T>,
}

impl<T> tonic::codec::Decoder for Decoder<T>
where
    T: for<'a> Deserialize<'a> + Unpin,
{
    type Item = T;
    type Error = tonic::Status;
    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut deserializer: tokio_serde::formats::Json<T, T> =
            tokio_serde::formats::Json::default();
        println!("{:?}", src);
        let mut bytes = bytes::BytesMut::new();
        bytes.extend_from_slice(&src.to_bytes());
        println!("{:?}", bytes);
        let result = Pin::new(&mut deserializer)
            .deserialize(&bytes)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))?;
        Ok(Some(result))
    }
}

pub struct Codec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> Default for Codec<T, U> {
    fn default() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U> tonic::codec::Codec for Codec<T, U>
where
    T: Send + Sync + Serialize + Unpin + 'static,
    U: Send + Sync + for<'a> Deserialize<'a> + Unpin + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = Encoder<T>;
    type Decoder = Decoder<U>;
    fn encoder(&mut self) -> Encoder<T> {
        Encoder { _pd: PhantomData }
    }
    fn decoder(&mut self) -> Decoder<U> {
        Decoder { _pd: PhantomData }
    }
}
