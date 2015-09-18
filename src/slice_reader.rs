use byteorder::{BigEndian, ReadBytesExt};
use std::mem;

pub enum Error {
   /// End of stream
   Eos,

   /// Invalid encoding
   Invalid(&'static str),

   /// Needs more data
   NeedMoreData(Option<usize>)
}

#[derive(Debug)]
pub enum Value<'a> {
    Nil,
    Boolean(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f32),
    Double(f64),
    String(&'a[u8]),
    Binary(&'a[u8]),
    Array(usize),
    Map(usize),
}

macro_rules! needs_more_data {
    (
        $n:expr, $data:expr
    ) => {
        if $data.len() < $n {
            return Err(Error::NeedMoreData(Some($n - $data.len())));
        } else {
            match $data.split_at($n) {
                (d, rest) => {
                    debug_assert!(d.len() == $n);
                    (d, rest)
                }
            }
        }
    }
}

macro_rules! be_u16 {
    (
        $item:expr
    ) => {
        match $item {
                mut item => {
                        debug_assert!(item.len() == 2);
                        let u = item.read_u16::<BigEndian>().unwrap();
                        debug_assert!(item.is_empty());
                        u
                }
        }
    }
}

macro_rules! be_u32 {
    (
        $item:expr
    ) => {
        match $item {
                mut item => {
                        debug_assert!(item.len() == 4);
                        let u = item.read_u32::<BigEndian>().unwrap();
                        debug_assert!(item.is_empty());
                        u
                }
        }
    }
}

macro_rules! be_u64 {
    (
        $item:expr
    ) => {
        match $item {
                mut item => {
                        debug_assert!(item.len() == 8);
                        let u = item.read_u64::<BigEndian>().unwrap();
                        debug_assert!(item.is_empty());
                        u
                }
        }
    }
}

pub fn parse_next<'a>(data: &'a[u8]) -> Result<(Value<'a>, &'a[u8]), Error> {
    match data.split_first() {
        Some((&c, rest)) => {
            match c {
                0xc0            => Ok((Value::Nil, rest)),
                0xc1            => Err(Error::Invalid("Reserved")),
                0xc2            => Ok((Value::Boolean(false), rest)),
                0xc3            => Ok((Value::Boolean(true), rest)),
                0x00 ... 0x7f   => Ok((Value::Unsigned(c as u64), rest)),


                //
                // Unsigned integers
                //

                0xcc            => match needs_more_data!(1, rest) {
                                        (item, rest) => Ok((Value::Unsigned(item[0] as u64), rest))
                                   },
                0xcd            => match needs_more_data!(2, rest) {
                                        (item, rest) => {
                                                Ok((Value::Unsigned(be_u16!(item) as u64), rest))
                                        }
                                   },
                0xce            => match needs_more_data!(4, rest) {
                                        (item, rest) => {
                                                Ok((Value::Unsigned(be_u32!(item) as u64), rest))
                                        }
                                   },
                0xcf            => match needs_more_data!(8, rest) {
                                        (item, rest) => {
                                                Ok((Value::Unsigned(be_u64!(item)), rest))
                                        }
                                   },

                //
                // Signed integers
                //

                0xd0            => match needs_more_data!(1, rest) {
                                        (item, rest) => Ok((Value::Signed((item[0] as i8) as i64), rest))
                                   },
                0xd1            => match needs_more_data!(2, rest) {
                                        (item, rest) => {
                                                Ok((Value::Signed((be_u16!(item) as i16) as i64), rest))
                                        }
                                   },
                0xd2            => match needs_more_data!(4, rest) {
                                        (item, rest) => {
                                                Ok((Value::Signed((be_u32!(item) as i32) as i64), rest))
                                        }
                                   },
                0xd3            => match needs_more_data!(8, rest) {
                                        (item, rest) => {
                                                Ok((Value::Signed(be_u64!(item) as i64), rest))
                                        }
                                   },
                0xe0 ... 0xff   => Ok((Value::Signed((c as i8) as i64), rest)),

                //
                // Floating point
                //

                0xca            => match needs_more_data!(4, rest) {
                                        (item, rest) => {
                                                let f: f32 = unsafe { mem::transmute(be_u32!(item)) };
                                                Ok((Value::Float(f), rest))
                                        }
                                   },
                0xcb            => match needs_more_data!(8, rest) {
                                        (item, rest) => {
                                                let d: f64 = unsafe { mem::transmute(be_u64!(item)) };
                                                Ok((Value::Double(d), rest))
                                        }
                                   },

                //
                // String
                //

                0xa0 ... 0xbf   => match needs_more_data!((c as usize) & 0x1F, rest) {
                                        (item, rest) => Ok((Value::String(item), rest))
                                   },

                0xd9            => match needs_more_data!(1, rest) {
                                        (item, rest) => match needs_more_data!(item[0] as usize, rest) {
                                                                (item, rest) => Ok((Value::String(item), rest))
                                                        }
                                   },

                0xda            => match needs_more_data!(2, rest) {
                                        (item, rest) => match needs_more_data!(be_u16!(item) as usize, rest) {
                                                                (item, rest) => Ok((Value::String(item), rest))
                                                        }
                                   },

                0xdb            => match needs_more_data!(4, rest) {
                                        (item, rest) => match needs_more_data!(be_u32!(item) as usize, rest) {
                                                                (item, rest) => Ok((Value::String(item), rest))
                                                        }
                                   },

                //
                // Binary
                //

                0xc4            => match needs_more_data!(1, rest) {
                                        (item, rest) => match needs_more_data!(item[0] as usize, rest) {
                                                                (item, rest) => Ok((Value::Binary(item), rest))
                                                        }
                                   },

                0xc5            => match needs_more_data!(2, rest) {
                                        (item, rest) => match needs_more_data!(be_u16!(item) as usize, rest) {
                                                                (item, rest) => Ok((Value::Binary(item), rest))
                                                        }
                                   },

                0xc6            => match needs_more_data!(4, rest) {
                                        (item, rest) => match needs_more_data!(be_u32!(item) as usize, rest) {
                                                                (item, rest) => Ok((Value::Binary(item), rest))
                                                        }
                                   },

                //
                // Array
                //

                0x90 ... 0x9f   => Ok((Value::Array((c as usize) & 0x0F), rest)),
                0xdc            => match needs_more_data!(2, rest) {
                                        (item, rest) => Ok((Value::Array(be_u16!(item) as usize), rest))
                                   },
                0xdd            => match needs_more_data!(4, rest) {
                                        (item, rest) => Ok((Value::Array(be_u32!(item) as usize), rest))
                                   },

                //
                // Map
                //

                0x80 ... 0x8f   => Ok((Value::Map((c as usize) & 0x0F), rest)),
                0xde            => match needs_more_data!(2, rest) {
                                        (item, rest) => Ok((Value::Map(be_u16!(item) as usize), rest))
                                   },
                0xdf            => match needs_more_data!(4, rest) {
                                        (item, rest) => Ok((Value::Map(be_u32!(item) as usize), rest))
                                   },

                // Ext: TODO

                 _               => Err(Error::Invalid("Invalid"))
            }
        }
        None => {
            Err(Error::Eos)
        }
    }
}