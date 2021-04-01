use std::marker::PhantomData;

use bytes::{buf::BufMut, Buf};
use tonic::Status;

pub trait SerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), tonic::Status>
    where
        T: serde::Serialize,
        W: std::io::Write;

    fn read<T, R>(r: R) -> Result<T, tonic::Status>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: std::io::Read;
}

#[derive(Clone, Copy)]
pub struct Encoder<C, T> {
    _pd: PhantomData<(C, T)>,
}

impl<C, T> tonic::codec::Encoder for Encoder<C, T>
where
    T: serde::Serialize,
    C: SerdeCodec,
{
    type Item = T;
    type Error = tonic::Status;
    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        C::write(item, dst.writer())
    }
}

#[derive(Clone, Copy)]
pub struct Decoder<C, T> {
    _pd: PhantomData<(C, T)>,
}

impl<C, T> tonic::codec::Decoder for Decoder<C, T>
where
    T: for<'de> serde::Deserialize<'de>,
    C: SerdeCodec,
{
    type Item = T;
    type Error = tonic::Status;
    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        Ok(Some(C::read::<T, _>(src.reader())?))
    }
}

pub struct Codec<C, T, U> {
    _pd: PhantomData<(C, T, U)>,
}

impl<C, T, U> Default for Codec<C, T, U> {
    fn default() -> Self {
        Codec { _pd: PhantomData }
    }
}

impl<C, T, U> tonic::codec::Codec for Codec<C, T, U>
where
    C: SerdeCodec + Send + Sync + 'static,
    T: serde::Serialize + Send + Sync + 'static,
    U: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = Encoder<C, T>;
    type Decoder = Decoder<C, U>;

    fn encoder(&mut self) -> Self::Encoder {
        Encoder { _pd: PhantomData }
    }

    fn decoder(&mut self) -> Self::Decoder {
        Decoder { _pd: PhantomData }
    }
}

pub struct BincodeSerdeCodec;
pub struct CborSerdeCodec;
pub struct JsonSerdeCodec;
pub struct MessagePackSerdeCodec;

impl SerdeCodec for BincodeSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: serde::Serialize,
        W: std::io::Write,
    {
        bincode::serialize_into(w, &item)
            .map_err(|bincode_err| Status::internal(format!("Error serializing {}", bincode_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: std::io::Read,
    {
        bincode::deserialize_from(r)
            .map_err(|bincode_err| Status::internal(format!("Error deserializing {}", bincode_err)))
    }
}

impl SerdeCodec for CborSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: serde::Serialize,
        W: std::io::Write,
    {
        serde_cbor::to_writer(w, &item)
            .map_err(|serde_err| Status::internal(format!("Error serializing {}", serde_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: std::io::Read,
    {
        serde_cbor::from_reader(r)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))
    }
}

impl SerdeCodec for JsonSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: serde::Serialize,
        W: std::io::Write,
    {
        serde_json::to_writer(w, &item)
            .map_err(|serde_err| Status::internal(format!("Error serializing {}", serde_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: std::io::Read,
    {
        serde_json::from_reader(r)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))
    }
}

impl SerdeCodec for MessagePackSerdeCodec {
    fn write<T, W>(item: T, mut w: W) -> Result<(), Status>
    where
        T: serde::Serialize,
        W: std::io::Write,
    {
        rmp_serde::encode::write(&mut w, &item).map_err(|message_pack_err| {
            Status::internal(format!("Error serializing {}", message_pack_err))
        })
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: std::io::Read,
    {
        rmp_serde::from_read(r).map_err(|message_pack_err| {
            Status::internal(format!("Error deserializing {}", message_pack_err))
        })
    }
}

pub type BincodeCodec<T, U> = Codec<BincodeSerdeCodec, T, U>;
pub type CborCodec<T, U> = Codec<CborSerdeCodec, T, U>;
pub type JsonCodec<T, U> = Codec<JsonSerdeCodec, T, U>;
pub type MessagePackCodec<T, U> = Codec<MessagePackSerdeCodec, T, U>;
