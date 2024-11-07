use crate::{util::write_slice, Bitset};

pub mod chars;

pub enum Message {
    FreeText(F71),
    DXpedition,
    FieldDay0,
    FieldDay1,
    Telemetry(T71),
    StdMsg {
        call1: C28,
        call1_r: bool,
        call2: C28,
        call2_r: bool,
        r: bool,
        grid: G15,
    },
    EuVhf {
        call1: C28,
        call1_p: bool,
        call2: C28,
        call2_p: bool,
        r: bool,
        grid: G15,
    },
    RttyRu,
    NonStdCall {
        hash: CallsignHash,
        call: C58,
        hash_is_second: bool,
        r: R2,
        cq: bool,
    },
    EuVhfHash,
}

macro_rules! writes {
    (
        $out:ident,
        $( $body: expr ),*
        $(,)?
    ) => {
        {
            let mut i_ = 0;
            $(
                i_ += macro_find_and_replace::replace_token!(_, (&mut $out[i_..]), $body)?;
            )*
            Some(i_)
        }
    };
}

impl Message {
    pub fn decode(bs: &Bitset) -> Option<Self> {
        let i3 = bs.slice(74, 3);
        match i3 {
            0 => {
                let n3 = bs.slice(71, 3);
                match n3 {
                    0 => Some(Self::FreeText(F71(bs.clone()))),
                    1 => Some(Self::DXpedition),
                    2 => Some(Self::FieldDay0),
                    3 => Some(Self::FieldDay1),
                    4 => Some(Self::Telemetry(T71(bs.clone()))),
                    // _ => panic!("invalid n3 value: {}", n3),
                    _ => None,
                }
            }
            1 => Some(Self::StdMsg {
                call1: C28(bs.slice(0, 28)),
                call1_r: bs.get(28),
                call2: C28(bs.slice(29, 28)),
                call2_r: bs.get(57),
                r: bs.get(58),
                grid: G15(bs.slice(59, 15) as u16),
            }),

            2 => Some(Self::EuVhf {
                call1: C28(bs.slice(0, 28)),
                call1_p: bs.get(28),
                call2: C28(bs.slice(29, 28)),
                call2_p: bs.get(57),
                r: bs.get(58),
                grid: G15(bs.slice(59, 15) as u16),
            }),
            3 => Some(Self::RttyRu),
            4 => Some(Self::NonStdCall {
                hash: CallsignHash::H12(bs.slice(0, 12) as u16),
                call: C58(bs.slice_u64(12, 58)),
                hash_is_second: bs.get(70),
                r: R2::from_val(bs.slice(71, 2) as u8),
                cq: bs.get(73),
            }),
            5 => Some(Self::EuVhfHash),
            // _ => panic!("invalid i3 value: {}", i3),
            _ => None,
        }
    }

    pub fn write_str(
        &self,
        out: &mut [u8],
        hashtable: Option<&impl CallsignHashTable>,
    ) -> Option<usize> {
        match self {
            Self::FreeText(f71) => f71.write_str(out),
            Self::DXpedition => {
                // K1ABC RR73; W9XYZ <KH1/KH7Z> -08
                write_slice(out, b"DXpedition")
            }
            Self::FieldDay0 => {
                // K1ABC W9XYZ 6A WI
                write_slice(out, b"FieldDay0")
            }
            Self::FieldDay1 => {
                // W9XYZ K1ABC R 17B EMA
                // out[..9].copy_from_slice(b"FieldDay1");
                write_slice(out, b"FieldDay1")
            }
            Self::Telemetry(_) => write_slice(out, b"Telemetry"),
            Self::StdMsg {
                call1,
                call1_r,
                call2,
                call2_r,
                r,
                grid,
            } => {
                // K1ABC/R W9XYZ/R R EN37
                writes! { out,
                    call1.write_str(_, hashtable),
                    if *call1_r { write_slice(_, b"/R") } else { Some(0) },
                    write_slice(_, b" "),

                    call2.write_str(_, hashtable),
                    if *call2_r { write_slice(_, b"/R") } else { Some(0) },
                    write_slice(_, b" "),

                    if *r { write_slice(_, b"R ") } else { Some(0) },
                    grid.write_str(_),
                }
            }
            Self::EuVhf {
                call1,
                call1_p,
                call2,
                call2_p,
                r,
                grid,
            } => {
                // G4ABC/P PA9XYZ JO22
                writes! { out,
                    call1.write_str(_, hashtable),
                    if *call1_p { write_slice(out, b"/P") } else { Some(0) },
                    write_slice(_, b" "),

                    call2.write_str(_, hashtable),
                    if *call2_p { write_slice(out, b"/P") } else { Some(0) },
                    write_slice(_, b" "),

                    if *r { write_slice(out, b"R ") } else { Some(0) },
                    grid.write_str(_),
                }
            }
            Self::RttyRu => {
                // K1ABC W9XYZ 579 WI
                write_slice(out, b"RttyRu")
            }
            Self::NonStdCall {
                cq,
                call,
                hash,
                hash_is_second,
                r,
                ..
            } => {
                // <W9XYZ> PJ4/K1ABC RRR
                if *cq {
                    writes! { out,
                        write_slice(_, b"CQ "),
                        call.write_str(_),
                    }
                } else if *hash_is_second {
                    writes! { out,
                        call.write_str(_),
                        write_slice(_, b" "),
                        hash.write_str(_, hashtable),
                        write_slice(_, b" "),
                        r.write_str(_),
                    }
                } else {
                    writes! { out,
                        hash.write_str(_, hashtable),
                        write_slice(_, b" "),
                        call.write_str(_),
                        write_slice(_, b" "),
                        r.write_str(_),
                    }
                }
            }
            Self::EuVhfHash => {
                // <G4ABC> <PA9XYZ> R 570007 JO22DB
                writes! { out,
                    write_slice(_, b"EuVhfHash"),
                }
            }
        }
    }

    pub fn encode(&self) -> Bitset {
        let mut ret = Bitset::default();

        match self {
            Message::FreeText(f71) => {
                ret = f71.0.clone();
                ret.set_slice(71, 3, 0); // FreeText
            }
            Message::Telemetry(t71) => {
                ret = t71.0.clone();
                ret.set_slice(71, 3, 4); // Telemetry
            }
            Message::StdMsg {
                call1,
                call1_r,
                call2,
                call2_r,
                r,
                grid,
            } => {
                ret.set_slice(0, 28, call1.0);
                ret.set(28, *call1_r);
                ret.set_slice(29, 28, call2.0);
                ret.set(57, *call2_r);
                ret.set(58, *r);
                ret.set_slice(59, 15, grid.0 as u32);
                ret.set_slice(74, 3, 1); // StdMsg
            }
            Message::EuVhf {
                call1,
                call1_p,
                call2,
                call2_p,
                r,
                grid,
            } => {
                ret.set_slice(0, 28, call1.0);
                ret.set(28, *call1_p);
                ret.set_slice(29, 28, call2.0);
                ret.set(57, *call2_p);
                ret.set(58, *r);
                ret.set_slice(59, 15, grid.0 as u32);

                ret.set_slice(74, 3, 2); // EuVhf
            }
            Message::NonStdCall {
                hash,
                call,
                hash_is_second,
                r,
                cq,
            } => {
                ret.set_slice(0, 12, hash.as_h12() as u32);
                // ret.set_slice(12, 58, call.0);
                ret.set_slice(12, 20, (call.0 >> 38) as u32);
                ret.set_slice(32, 32, (call.0 >> 6) as u32);
                ret.set_slice(64, 6, (call.0 & 0x3F) as u32);
                ret.set(70, *hash_is_second);
                ret.set_slice(71, 2, r.to_val() as u32);
                ret.set(73, *cq);

                ret.set_slice(74, 3, 4); // NonStdCall
            }
            _ => {} // not implemented but no error
        }
        ret
    }
}

#[cfg(not(feature = "no_std"))]
impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut s = [0; 64];
        let n = self.write_str(&mut s, None::<&()>).unwrap();
        f.write_str(core::str::from_utf8(&s[..n]).unwrap())
    }
}

pub mod callsign;
use callsign::{
    hash::{CallsignHash, CallsignHashTable},
    C28, C58,
};

mod grid15;
pub use grid15::G15;

mod roger2;
pub use roger2::R2;

mod freetext;
pub use freetext::F71;

// TODO: implement these mocks
pub struct T71(Bitset); // telemetry data

// TODO: implement remaining types; now only frequently used types are implemented
// pub struct G25(u32); // grid locator 6
// pub struct H10(u16); // hash
// pub struct H22(u32); // hash
// pub struct K3(u8); // 3 bits
// pub struct N4; // transmission number
// pub struct R3; // signal report
// pub struct R5; // signal report in dB
// pub struct Roger1; // Roger flag
// pub struct S11; // sreial number
// pub struct S13; // serial number
// pub struct S7; // section name
