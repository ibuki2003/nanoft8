use crate::Bitset;

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
        hash: H12,
        call: C58,
        hash_is_second: bool,
        r: R2,
        cq: bool,
    },
    EuVhfHash,
}

macro_rules! writes_str {
    (
        $out:ident;
        $( $len:expr => $body: stmt );*
        $(;)?
    ) => {
        {
            let mut i_ = 0;
            $(
                macro_find_and_replace::replace_token!(_, (&mut $out[i_..i_+$len]), $body);
                i_ += $len;
            )*
            i_
        };
    };
}

impl Message {
    pub fn decode(bs: &Bitset) -> Result<Self, ()> {
        let i3 = bs.slice(74, 3);
        match i3 {
            0 => {
                let n3 = bs.slice(71, 3);
                match n3 {
                    0 => Ok(Self::FreeText(F71(bs.clone()))),
                    1 => Ok(Self::DXpedition),
                    2 => Ok(Self::FieldDay0),
                    3 => Ok(Self::FieldDay1),
                    4 => Ok(Self::Telemetry(T71(bs.clone()))),
                    // _ => panic!("invalid n3 value: {}", n3),
                    _ => Err(()),
                }
            }
            1 => Ok(Self::StdMsg {
                call1: C28(bs.slice(0, 28)),
                call1_r: bs.get(28),
                call2: C28(bs.slice(29, 28)),
                call2_r: bs.get(57),
                r: bs.get(58),
                grid: G15(bs.slice(59, 15) as u16),
            }),

            2 => Ok(Self::EuVhf {
                call1: C28(bs.slice(0, 28)),
                call1_p: bs.get(28),
                call2: C28(bs.slice(29, 28)),
                call2_p: bs.get(57),
                r: bs.get(58),
                grid: G15(bs.slice(59, 15) as u16),
            }),
            3 => Ok(Self::RttyRu),
            4 => Ok(Self::NonStdCall {
                hash: H12(bs.slice(0, 12) as u16),
                call: C58(bs.slice_u64(12, 58)),
                hash_is_second: bs.get(70),
                r: R2(bs.slice(71, 2) as u8),
                cq: bs.get(73),
            }),
            5 => Ok(Self::EuVhfHash),
            // _ => panic!("invalid i3 value: {}", i3),
            _ => Err(()),
        }
    }

    pub fn to_string(&self, out: &mut [u8]) {
        out.fill(b' ');

        match self {
            Self::FreeText(_) => {
                out[..8].copy_from_slice(b"FreeText");
            }
            Self::DXpedition => {
                // K1ABC RR73; W9XYZ <KH1/KH7Z> -08
                out[..10].copy_from_slice(b"DXpedition");
            }
            Self::FieldDay0 => {
                // K1ABC W9XYZ 6A WI
                out[..9].copy_from_slice(b"FieldDay0");
            }
            Self::FieldDay1 => {
                // W9XYZ K1ABC R 17B EMA
                out[..9].copy_from_slice(b"FieldDay1");
            }
            Self::Telemetry(_) => {
                out[..9].copy_from_slice(b"Telemetry");
            }
            Self::StdMsg {
                call1,
                call1_r,
                call2,
                call2_r,
                r,
                grid,
            } => {
                // // K1ABC/R W9XYZ/R R EN37
                writes_str! { out;
                    7 => _.copy_from_slice(b"StdMsg ");
                    7 => call1.to_string(_);
                    2 => if *call1_r { _.copy_from_slice(b"/R") };
                    1 => {};
                    7 => call2.to_string(_);
                    2 => if *call2_r { _.copy_from_slice(b"/R") };
                    1 => {};
                    1 => if *r { _.copy_from_slice(b"R") };
                    1 => {};
                    4 => grid.to_string(_);
                };
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
                writes_str! { out;
                    7 => _.copy_from_slice(b"EuVhf  ");
                    7 => call1.to_string(_);
                    2 => if *call1_p { _.copy_from_slice(b"/P") };
                    1 => {};
                    7 => call2.to_string(_);
                    2 => if *call2_p { _.copy_from_slice(b"/P") };
                    1 => {};
                    1 => if *r { _.copy_from_slice(b"R") };
                    1 => {};
                    4 => grid.to_string(_);
                };

            }
            Self::RttyRu => {
                // K1ABC W9XYZ 579 WI
                writes_str! { out;
                    6 => _.copy_from_slice(b"RttyRu");
                };
            }
            Self::NonStdCall { .. } => {
                // <W9XYZ> PJ4/K1ABC RRR
                writes_str! { out;
                    10 => _.copy_from_slice(b"NonStdCall");
                };

            }
            Self::EuVhfHash => {
                // <G4ABC> <PA9XYZ> R 570007 JO22DB
                writes_str! { out;
                    9 => _.copy_from_slice(b"EuVhfHash");
                };
            }
        }
    }
}

mod callsign28;
pub use callsign28::C28;

mod grid15;
pub use grid15::G15;

// TODO: implement these mocks
pub struct C58(u64); // NonStdCall
pub struct F71(Bitset); // free text
pub struct H12(u16); // hash
pub struct R2(u8); // RRR message
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

