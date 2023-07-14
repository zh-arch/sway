use crate::{type_system::*, Engines};

use num_bigint::BigUint;
use sway_error::error::CompileError;
use sway_types::{integer_bits::IntegerBits, span};

use std::{
    fmt,
    hash::{Hash, Hasher},
    num::IntErrorKind,
};

#[derive(Debug, Clone, Eq)]
pub struct U256 {
    value: BigUint,
}

impl PartialEq for U256 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl U256 {
    pub fn min() -> U256 {
        let value = BigUint::from_bytes_le(&[0]);
        U256 { value }
    }

    pub fn max() -> U256 {
        let value = BigUint::from_bytes_le(&[255; 32]);
        U256 { value }
    }
}

impl TryFrom<BigUint> for U256 {
    type Error = IntErrorKind;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let mut bytes = value.to_bytes_le();

        // Normalize removing zeros from the most signicants positions
        if let Some(&0) = bytes.last() {
            let len = bytes.iter().rposition(|&d| d != 0).map_or(0, |i| i + 1);
            bytes.truncate(len);
        }

        if bytes.len() <= 32 {
            Ok(U256 {
                value: BigUint::from_bytes_le(&bytes),
            })
        } else {
            Err(IntErrorKind::PosOverflow)
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
    String(span::Span),
    Numeric(BigUint),
    Boolean(bool),
    B256([u8; 32]),
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Literal::*;
        match self {
            U8(x) => {
                state.write_u8(1);
                x.hash(state);
            }
            U16(x) => {
                state.write_u8(2);
                x.hash(state);
            }
            U32(x) => {
                state.write_u8(3);
                x.hash(state);
            }
            U64(x) => {
                state.write_u8(4);
                x.hash(state);
            }
            U128(_) => todo!(),
            U256(_) => todo!(),
            Numeric(x) => {
                state.write_u8(5);
                x.hash(state);
            }
            String(inner) => {
                state.write_u8(6);
                inner.as_str().hash(state);
            }
            Boolean(x) => {
                state.write_u8(7);
                x.hash(state);
            }
            B256(x) => {
                state.write_u8(8);
                x.hash(state);
            }
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::U128(l0), Self::U128(r0)) => l0 == r0,
            (Self::U256(l0), Self::U256(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => *l0.as_str() == *r0.as_str(),
            (Self::Numeric(l0), Self::Numeric(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::B256(l0), Self::B256(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Literal::U8(content) => content.to_string(),
            Literal::U16(content) => content.to_string(),
            Literal::U32(content) => content.to_string(),
            Literal::U64(content) => content.to_string(),
            Literal::U128(_) => todo!(),
            Literal::U256(_) => todo!(),
            Literal::Numeric(content) => content.to_string(),
            Literal::String(content) => content.as_str().to_string(),
            Literal::Boolean(content) => content.to_string(),
            Literal::B256(content) => content
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        };
        write!(f, "{s}")
    }
}

impl Literal {
    #[allow(clippy::wildcard_in_or_patterns)]
    pub(crate) fn handle_parse_int_error(
        engines: &Engines,
        e: &IntErrorKind,
        ty: TypeInfo,
        span: sway_types::Span,
    ) -> CompileError {
        match e {
            IntErrorKind::PosOverflow => CompileError::IntegerTooLarge {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::NegOverflow => CompileError::IntegerTooSmall {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::InvalidDigit => CompileError::IntegerContainsInvalidDigit {
                ty: engines.help_out(ty).to_string(),
                span,
            },
            IntErrorKind::Zero | IntErrorKind::Empty | _ => {
                CompileError::Internal("Called incorrect internal sway-core on literal type.", span)
            }
        }
    }

    pub(crate) fn to_typeinfo(&self) -> TypeInfo {
        match self {
            Literal::String(s) => TypeInfo::Str(Length::new(s.as_str().len(), s.clone())),
            Literal::Numeric(_) => TypeInfo::Numeric,
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::U128(_) => TypeInfo::UnsignedInteger(IntegerBits::V128),
            Literal::U256(_) => TypeInfo::UnsignedInteger(IntegerBits::V256),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::B256(_) => TypeInfo::B256,
        }
    }
}
