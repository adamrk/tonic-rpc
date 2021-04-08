use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use bytes::{buf::BufMut, Buf};
use serde::{Deserialize, Serialize};
use tonic::{codec, Status};

pub trait SerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: Serialize,
        W: Write;

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> Deserialize<'de>,
        R: Read;
}

#[derive(Clone, Copy)]
pub struct Encoder<C, T> {
    _pd: PhantomData<(C, T)>,
}

impl<C, T> codec::Encoder for Encoder<C, T>
where
    T: Serialize,
    C: SerdeCodec,
{
    type Item = T;
    type Error = Status;
    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        C::write(item, dst.writer())
    }
}

#[derive(Clone, Copy)]
pub struct Decoder<C, T> {
    _pd: PhantomData<(C, T)>,
}

impl<C, T> codec::Decoder for Decoder<C, T>
where
    T: for<'de> Deserialize<'de>,
    C: SerdeCodec,
{
    type Item = T;
    type Error = Status;
    fn decode(
        &mut self,
        src: &mut codec::DecodeBuf<'_>,
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

impl<C, T, U> codec::Codec for Codec<C, T, U>
where
    C: SerdeCodec + Send + Sync + 'static,
    T: Serialize + Send + Sync + 'static,
    U: for<'de> Deserialize<'de> + Send + Sync + 'static,
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

#[cfg(feature = "bincode")]
#[cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
pub struct BincodeSerdeCodec;
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub struct CborSerdeCodec;
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub struct JsonSerdeCodec;
#[cfg(feature = "messagepack")]
#[cfg_attr(docsrs, doc(cfg(feature = "messagepack")))]
pub struct MessagePackSerdeCodec;

#[cfg(feature = "bincode")]
#[cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
impl SerdeCodec for BincodeSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: Serialize,
        W: Write,
    {
        bincode::serialize_into(w, &item)
            .map_err(|bincode_err| Status::internal(format!("Error serializing {}", bincode_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> Deserialize<'de>,
        R: Read,
    {
        bincode::deserialize_from(r)
            .map_err(|bincode_err| Status::internal(format!("Error deserializing {}", bincode_err)))
    }
}

#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
impl SerdeCodec for CborSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: Serialize,
        W: Write,
    {
        serde_cbor::to_writer(w, &item)
            .map_err(|serde_err| Status::internal(format!("Error serializing {}", serde_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> Deserialize<'de>,
        R: Read,
    {
        serde_cbor::from_reader(r)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl SerdeCodec for JsonSerdeCodec {
    fn write<T, W>(item: T, w: W) -> Result<(), Status>
    where
        T: Serialize,
        W: Write,
    {
        serde_json::to_writer(w, &item)
            .map_err(|serde_err| Status::internal(format!("Error serializing {}", serde_err)))
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> Deserialize<'de>,
        R: Read,
    {
        serde_json::from_reader(r)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))
    }
}

#[cfg(feature = "messagepack")]
#[cfg_attr(docsrs, doc(cfg(feature = "messagepack")))]
impl SerdeCodec for MessagePackSerdeCodec {
    fn write<T, W>(item: T, mut w: W) -> Result<(), Status>
    where
        T: Serialize,
        W: Write,
    {
        rmp_serde::encode::write(&mut w, &item).map_err(|message_pack_err| {
            Status::internal(format!("Error serializing {}", message_pack_err))
        })
    }

    fn read<T, R>(r: R) -> Result<T, Status>
    where
        T: for<'de> Deserialize<'de>,
        R: Read,
    {
        rmp_serde::from_read(r).map_err(|message_pack_err| {
            Status::internal(format!("Error deserializing {}", message_pack_err))
        })
    }
}

#[cfg(feature = "bincode")]
#[cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
pub type BincodeCodec<T, U> = Codec<BincodeSerdeCodec, T, U>;
#[cfg(feature = "cbor")]
#[cfg_attr(docsrs, doc(cfg(feature = "cbor")))]
pub type CborCodec<T, U> = Codec<CborSerdeCodec, T, U>;
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub type JsonCodec<T, U> = Codec<JsonSerdeCodec, T, U>;
#[cfg(feature = "messagepack")]
#[cfg_attr(docsrs, doc(cfg(feature = "messagepack")))]
pub type MessagePackCodec<T, U> = Codec<MessagePackSerdeCodec, T, U>;
