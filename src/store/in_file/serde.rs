// EXPERIMENT //

#[derive(Debug, thiserror::Error)]
#[error("Oh nei!")]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("Unknown: {0}")]
    Unknown(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

pub struct Serde<'w, W: std::io::Write> {
    writer: &'w mut W,
    filled: bool,
    depth: usize,
}

impl<'w, W: std::io::Write> Serde<'w, W> {
    pub fn new(writer: &'w mut W) -> Self {
        Self {
            writer,
            filled: false,
            depth: 0,
        }
    }

    fn step_in(&mut self) -> &mut Self {
        self.depth += 1;
        self
    }

    fn separate(&mut self) -> Result<(), Error> {
        if self.filled {
            self.writer.write_all(b"\t")?;
        } else {
            self.filled = true;
        }

        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::Serializer for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        if value {
            self.writer.write_all(b"true")?;
        } else {
            self.writer.write_all(b"false")?;
        }
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(ryu::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(ryu::Buffer::new().format(value).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(value.encode_utf8(&mut [0; 4]).as_bytes())
            .map_err(Into::into)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(value.as_bytes()).map_err(Into::into)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(value).map_err(Into::into)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + serde::ser::Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(variant.as_bytes())
            .map_err(Into::into)
    }

    fn serialize_newtype_struct<T: ?Sized + serde::ser::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + serde::ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self.step_in())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self.step_in())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self.step_in())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.writer.write_all(variant.as_bytes())?;
        self.filled = true;
        Ok(self.step_in())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self.step_in())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self.step_in())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.writer.write_all(variant.as_bytes())?;
        self.filled = true;
        Ok(self.step_in())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeSeq for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeTuple for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeTupleStruct for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeTupleVariant for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeMap for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, _key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeStruct for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<'w, W: std::io::Write> serde::ser::SerializeStructVariant for &mut Serde<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.separate()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.depth -= 1;
        if self.depth == 0 {
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::{Skull, WithId};
    use super::Serde;

    #[test]
    fn serialize_flat() {
        let skull = WithId::new(
            2,
            Skull {
                name: String::from("xnamex"),
                color: String::from("xcolorx"),
                icon: String::from("xiconx"),
                unit_price: 0.1,
                limit: None,
            },
        );

        let mut buffer = vec![];
        let mut serder = Serde::new(&mut buffer);

        serde::Serialize::serialize(&skull, &mut serder).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output.as_str(), "2\txnamex\txcolorx\txiconx\t0.1\t\n");
    }

    #[test]
    fn serialize_nested() {
        #[derive(serde::Serialize, Debug)]
        struct Inner {
            f: f64,
            b: bool,
            i: i32,
        }

        #[derive(serde::Serialize, Debug)]
        struct Outer {
            foo: String,
            bla: Option<Inner>,
            kool: Option<Inner>,
        }

        let payload = Outer {
            foo: String::from("bar"),
            bla: None,
            kool: Some(Inner {
                f: 1.0,
                b: false,
                i: -3,
            }),
        };

        let mut buffer = vec![];
        let mut serder = Serde::new(&mut buffer);

        serde::Serialize::serialize(&payload, &mut serder).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output.as_str(), "bar\t\t1.0\tfalse\t-3\n");
    }
}
