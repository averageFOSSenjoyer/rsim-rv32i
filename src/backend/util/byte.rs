#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(dead_code)]

use crate::backend::util::event::{ByteEvent, WordEvent};
use crate::backend::util::types::{Byte, Word};
use rsim_core::event::{Event, EventValue};
use rsim_core::types::{Cycle, EventId};
use std::cmp::Ordering;
use std::fmt::{Binary, Debug, Display, Formatter, LowerHex, UpperHex};
use std::ops::{Index, IndexMut};
use std::option::Option;

/// A generic Byte type
///
/// Bytes[0] is the **least** significant byte
///
/// Due to generic limitations (pending stable generic_const_exprs),
/// operation will return Bytes of the same size as lhs, as opposed to the bigger of the two.
///
/// Support operations
/// - bytes = lhs + rhs
/// - lhs += rhs
/// - bytes = lhs - rhs
/// - lhs -= rhs
/// - bytes = lhs & rhs
/// - lhs &= rhs
/// - bytes = lhs | rhs
/// - lhs |= rhs
/// - bytes = lhs ^ rhs
/// - lhs ^= rhs
/// - lhs << rhs
/// - lhs <<= rhs
/// - lhs >> rhs
/// - lhs >>= rhs
///
/// Note:
/// - DO NOT use the derived Ord for comparison, use either `byte_cmp` or `signed_cmp`
/// - Left/Right shifting assumes the rhs is within usize, if you need to shift more than 8^64, you have better things to use than this simulator
/// - Bytes are treated as **unsigned**
/// - Operation will have funky result if unknowns exist in add/sub operands
///
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bytes<const T: usize> {
    /// The container for the bytes, exposed for easier manipulation.
    /// Users should prefer the implemented traits over directly manipulating the container.
    pub data: [Option<u8>; T],
}

impl<const T: usize> Bytes<T> {
    pub fn unknown() -> Self {
        Bytes { data: [None; T] }
    }
    pub fn zeros() -> Self {
        Bytes { data: [Some(0); T] }
    }
    pub fn has_unknown(&self) -> bool {
        for byte in self.data {
            if byte.is_none() {
                return true;
            }
        }
        false
    }
    pub fn is_zero(&self) -> bool {
        for byte in self.data {
            if byte.is_some() && byte.unwrap() != 0 {
                return false;
            }
        }
        true
    }
    pub fn is_something_nonzero(&self) -> bool {
        !self.has_unknown() && !self.is_zero()
    }
}

impl EventValue for Word {
    fn build_event(&self, event_id: EventId, scheduled_time: Cycle) -> Box<dyn Event> {
        Box::new(WordEvent::new(scheduled_time, *self, event_id))
    }
}

impl EventValue for Byte {
    fn build_event(&self, event_id: EventId, scheduled_time: Cycle) -> Box<dyn Event> {
        Box::new(ByteEvent::new(scheduled_time, *self, event_id))
    }
}

impl<const T: usize> Default for Bytes<T> {
    fn default() -> Self {
        Bytes::unknown()
    }
}

impl<const T: usize> Display for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .data
            .iter()
            .rev()
            .map(|x| {
                x.map(|byte| format!("{:02X}", byte))
                    .unwrap_or("XX".to_string())
            })
            .collect::<String>();
        write!(f, "0x{}", value)
    }
}

impl<const T: usize> LowerHex for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .data
            .iter()
            .rev()
            .map(|x| {
                x.map(|byte| format!("{:02x}", byte))
                    .unwrap_or("xx".to_string())
            })
            .collect::<String>();
        write!(f, "{}", value)
    }
}

impl<const T: usize> UpperHex for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .data
            .iter()
            .rev()
            .map(|x| {
                x.map(|byte| format!("{:02X}", byte))
                    .unwrap_or("XX".to_string())
            })
            .collect::<String>();
        write!(f, "{}", value)
    }
}

impl<const T: usize> Binary for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .data
            .iter()
            .rev()
            .map(|x| {
                x.map(|byte| format!("{:b}", byte))
                    .unwrap_or("xxxxxxxx".to_string())
            })
            .collect::<String>();
        write!(f, "{}", value)
    }
}

impl<const T: usize> Debug for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl<const T: usize> Index<usize> for Bytes<T> {
    type Output = Option<u8>;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<const T: usize> IndexMut<usize> for Bytes<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

pub trait ByteOrd<Rhs = Self> {
    fn byte_cmp(self, rhs: Rhs) -> Ordering;
}
impl<const T: usize, const R: usize> ByteOrd<Bytes<R>> for Bytes<T> {
    fn byte_cmp(self, other: Bytes<R>) -> Ordering {
        if self.has_unknown() || other.has_unknown() {
            return Ordering::Equal;
        }

        // if self is longer than other
        for i in R..T {
            if self[i].unwrap() > 0 {
                return Ordering::Greater;
            }
        }

        // if other is longer than self
        for i in T..R {
            if other[i].unwrap() > 0 {
                return Ordering::Less;
            }
        }

        // they are the same length
        for i in (0..T).rev() {
            match self[i].unwrap().cmp(&other[i].unwrap()) {
                Ordering::Greater => return Ordering::Greater,
                Ordering::Less => return Ordering::Less,
                _ => {}
            }
        }

        Ordering::Equal
    }
}

pub trait SignedOrd<Rhs = Self> {
    fn signed_cmp(self, rhs: Rhs) -> Ordering;
}

impl<const T: usize, const R: usize> SignedOrd<Bytes<R>> for Bytes<T> {
    fn signed_cmp(self, rhs: Bytes<R>) -> Ordering {
        if self.has_unknown() || rhs.has_unknown() {
            return Ordering::Equal;
        }

        let self_is_negative = (self[T - 1].unwrap() >> 7) & 0x1 == 0x1;
        let other_is_negative = (rhs[R - 1].unwrap() >> 7) & 0x1 == 0x1;

        if self_is_negative != other_is_negative {
            return if self_is_negative && !other_is_negative {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let mut self_no_msb = self;
        self_no_msb[T - 1] = Some(self_no_msb[T - 1].unwrap() & 0x7F);
        let mut other_no_msb = rhs;
        other_no_msb[T - 1] = Some(other_no_msb[T - 1].unwrap() & 0x7F);

        // if self is longer than other
        for i in R..T {
            if self_no_msb[i].unwrap() > 0 {
                return if self_is_negative {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
        }

        // if other is longer than self
        for i in T..R {
            if other_no_msb[i].unwrap() > 0 {
                return if self_is_negative {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
        }

        self.byte_cmp(rhs)
    }
}

impl<const T: usize> From<u8> for Bytes<T> {
    fn from(value: u8) -> Self {
        assert!(T >= 1);
        let mut result = Bytes::unknown();
        for i in 0..1 {
            result.data[i] = Some(value);
        }
        result
    }
}

impl<const T: usize> From<u16> for Bytes<T> {
    fn from(value: u16) -> Self {
        assert!(T >= 2);
        let mut result = Bytes::unknown();
        for i in 0..2 {
            result.data[i] = Some(((value >> (8 * i)) & 0xFF) as u8);
        }
        result
    }
}

impl<const T: usize> From<u32> for Bytes<T> {
    fn from(value: u32) -> Self {
        assert!(T >= 4);
        let mut result = Bytes::unknown();
        for i in 0..4 {
            result.data[i] = Some(((value >> (8 * i)) & 0xFF) as u8);
        }
        result
    }
}

impl<const T: usize> From<u64> for Bytes<T> {
    fn from(value: u64) -> Self {
        assert!(T >= 8);
        let mut result = Bytes::unknown();
        for i in 0..8 {
            result.data[i] = Some(((value >> (8 * i)) & 0xFF) as u8);
        }
        result
    }
}

impl<const T: usize> From<u128> for Bytes<T> {
    fn from(value: u128) -> Self {
        assert!(T >= 16);
        let mut result = Bytes::unknown();
        for i in 0..16 {
            result.data[i] = Some(((value >> (8 * i)) & 0xFF) as u8);
        }
        result
    }
}

impl<const T: usize> From<Bytes<T>> for Option<u8> {
    fn from(val: Bytes<T>) -> Self {
        assert!(T >= 1);
        val.data[0]
    }
}

impl<const T: usize> From<Bytes<T>> for Option<u16> {
    fn from(val: Bytes<T>) -> Self {
        let mut ret = Some(0);
        for i in 0..T.min(2) {
            if val.data[i].is_none() {
                ret = None;
                break;
            } else {
                ret = Some(ret.unwrap() | ((val.data[i].unwrap() as u16) << (i * 8)));
            }
        }
        ret
    }
}

impl<const T: usize> From<Bytes<T>> for Option<u32> {
    fn from(val: Bytes<T>) -> Self {
        let mut ret = Some(0);
        for i in 0..T.min(4) {
            if val.data[i].is_none() {
                ret = None;
                break;
            } else {
                ret = Some(ret.unwrap() | ((val.data[i].unwrap() as u32) << (i * 8)));
            }
        }
        ret
    }
}

impl<const T: usize> From<Bytes<T>> for Option<u64> {
    fn from(val: Bytes<T>) -> Self {
        let mut ret = Some(0);
        for i in 0..T.min(8) {
            if val.data[i].is_none() {
                ret = None;
                break;
            } else {
                ret = Some(ret.unwrap() | ((val.data[i].unwrap() as u64) << (i * 8)));
            }
        }
        ret
    }
}

impl<const T: usize> From<Bytes<T>> for Option<u128> {
    fn from(val: Bytes<T>) -> Self {
        let mut ret = Some(0);
        for i in 0..T.min(16) {
            if val.data[i].is_none() {
                ret = None;
                break;
            } else {
                ret = Some(ret.unwrap() | ((val.data[i].unwrap() as u128) << (i * 8)));
            }
        }
        ret
    }
}

impl<const T: usize, const R: usize> std::ops::Add<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn add(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = self;

        let mut extended_rhs = Bytes::<T>::unknown();
        for i in 0..T {
            extended_rhs.data[i] = if i < R { rhs.data[i] } else { Some(0x00) }
        }

        let mut cout: u8 = 0;
        for i in 0..T {
            result.data[i] = if let Some(lhs_byte) = result.data[i] {
                extended_rhs.data[i].map(|rhs_byte| {
                    let imm_add = lhs_byte as u16 + rhs_byte as u16 + cout as u16;
                    cout = (imm_add >> 8) as u8;
                    imm_add as u8
                })
            } else {
                cout = 0;
                None
            }
        }

        result
    }
}

impl<const T: usize, const R: usize> std::ops::AddAssign<Bytes<R>> for Bytes<T> {
    fn add_assign(&mut self, rhs: Bytes<R>) {
        *self = *self + rhs;
    }
}

impl<const T: usize, const R: usize> std::ops::Sub<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn sub(self, rhs: Bytes<R>) -> Self::Output {
        let mut extended_rhs = Bytes::<T>::unknown();
        for i in 0..T {
            extended_rhs.data[i] = if i < R { rhs.data[i] } else { Some(0x00) }
        }

        self + !extended_rhs + Bytes::<1>::from(1u8)
    }
}

impl<const T: usize, const R: usize> std::ops::SubAssign<Bytes<R>> for Bytes<T> {
    fn sub_assign(&mut self, rhs: Bytes<R>) {
        *self = *self - rhs;
    }
}

impl<const T: usize, const R: usize> std::ops::BitAnd<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn bitand(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = self;

        let mut extended_rhs = Bytes::<T>::unknown();
        for i in 0..T {
            extended_rhs.data[i] = if i < R { rhs.data[i] } else { Some(0x00) }
        }

        let _: Vec<_> = (0..T)
            .map(|i| {
                result.data[i] = match (result.data[i], extended_rhs.data[i]) {
                    (Some(0), _) => Some(0),
                    (_, Some(0)) => Some(0),
                    (Some(lhs), Some(rhs)) => Some(lhs & rhs),
                    _ => None,
                }
            })
            .collect();

        result
    }
}

impl<const T: usize, const R: usize> std::ops::BitAndAssign<Bytes<R>> for Bytes<T> {
    fn bitand_assign(&mut self, rhs: Bytes<R>) {
        *self = *self & rhs;
    }
}

impl<const T: usize, const R: usize> std::ops::BitOr<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn bitor(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = self;

        let _: Vec<_> = (0..R)
            .filter(|i| i < &T)
            .map(|i| {
                result.data[i] = match (result.data[i], rhs.data[i]) {
                    (Some(0xFF), _) => Some(0xFF),
                    (_, Some(0xFF)) => Some(0xFF),
                    (Some(lhs), Some(rhs)) => Some(lhs | rhs),
                    _ => None,
                }
            })
            .collect();

        result
    }
}

impl<const T: usize, const R: usize> std::ops::BitOrAssign<Bytes<R>> for Bytes<T> {
    fn bitor_assign(&mut self, rhs: Bytes<R>) {
        *self = *self | rhs;
    }
}

impl<const T: usize, const R: usize> std::ops::BitXor<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn bitxor(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = self;

        let _: Vec<_> = (0..R)
            .filter(|i| i < &T)
            .map(|i| {
                result.data[i] = match (result.data[i], rhs.data[i]) {
                    (Some(0xFF), Some(v)) => Some(!v),
                    (Some(v), Some(0xFF)) => Some(!v),
                    (Some(lhs), Some(rhs)) => Some(lhs ^ rhs),
                    _ => None,
                }
            })
            .collect();

        result
    }
}

impl<const T: usize, const R: usize> std::ops::BitXorAssign<Bytes<R>> for Bytes<T> {
    fn bitxor_assign(&mut self, rhs: Bytes<R>) {
        *self = *self ^ rhs;
    }
}

impl<const T: usize> std::ops::Not for Bytes<T> {
    type Output = Bytes<T>;

    fn not(self) -> Self::Output {
        let mut result = self;

        let _: Vec<_> = (0..T)
            .map(|i| self.data[i].map(|byte| result.data[i] = Some(!byte)))
            .collect();

        result
    }
}

impl<const T: usize, const R: usize> std::ops::Shl<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn shl(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = Self::zeros();

        if self.has_unknown() || rhs.has_unknown() {
            return Self::unknown();
        }

        let shift_count = Into::<Option<u128>>::into(rhs).unwrap() as usize;
        let byte_shift_count = shift_count / 8;
        let bit_shift_count = shift_count % 8;
        if byte_shift_count > T {
            return Self::zeros();
        }

        for i in byte_shift_count..T {
            result.data[i] = self.data[i - byte_shift_count];
        }

        let mut shift_in = 0u8;
        for i in 0..T {
            let byte = ((result.data[i].unwrap() as u16) << bit_shift_count) | (shift_in as u16);
            shift_in = (byte >> 8) as u8;
            result.data[i] = Some(byte as u8);
        }

        result
    }
}

impl<const T: usize, const R: usize> std::ops::ShlAssign<Bytes<R>> for Bytes<T> {
    fn shl_assign(&mut self, rhs: Bytes<R>) {
        *self = *self << rhs;
    }
}

impl<const T: usize, const R: usize> std::ops::Shr<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;
    fn shr(self, rhs: Bytes<R>) -> Self::Output {
        let mut result = Self::zeros();

        if self.has_unknown() || rhs.has_unknown() {
            return Self::unknown();
        }

        let shift_count = Into::<Option<u128>>::into(rhs).unwrap() as usize;
        let byte_shift_count = shift_count / 8;
        let bit_shift_count = shift_count % 8;
        if byte_shift_count > T {
            return Self::zeros();
        }

        for i in 0..T - byte_shift_count {
            result.data[i] = self.data[i + byte_shift_count];
        }

        let mut shift_in = 0u8;
        for i in (0..T).rev() {
            let byte = (result.data[i].unwrap() as u16) | ((shift_in as u16) << 8);
            shift_in = (byte & (0xFF >> (8 - bit_shift_count))) as u8;
            result.data[i] = Some((byte >> bit_shift_count) as u8);
        }

        result
    }
}

impl<const T: usize, const R: usize> std::ops::ShrAssign<Bytes<R>> for Bytes<T> {
    fn shr_assign(&mut self, rhs: Bytes<R>) {
        *self = *self >> rhs;
    }
}

pub trait Shra<Rhs = Self> {
    type Output;
    fn shra(self, rhs: Rhs) -> Self::Output;
}

impl<const T: usize, const R: usize> Shra<Bytes<R>> for Bytes<T> {
    type Output = Bytes<T>;

    fn shra(self, rhs: Bytes<R>) -> Bytes<T> {
        let mut result = Self::zeros();

        if self.has_unknown() || rhs.has_unknown() {
            return Self::unknown();
        }

        let msb_fill = if (self.data[T - 1].unwrap() >> 7) & 0x1 == 0x1 {
            Some(0xFF)
        } else {
            Some(0x00)
        };

        let shift_count = Into::<Option<u128>>::into(rhs).unwrap() as usize;
        let byte_shift_count = shift_count / 8;
        let bit_shift_count = shift_count % 8;
        if byte_shift_count > T {
            return Self::zeros();
        }

        for i in 0..T - byte_shift_count {
            result.data[i] = self.data[i + byte_shift_count];
        }

        let mut shift_in = 0u8;
        for i in (0..T).rev() {
            let byte = (result.data[i].unwrap() as u16) | ((shift_in as u16) << 8);
            shift_in = (byte & (0xFF >> (8 - bit_shift_count))) as u8;
            result.data[i] = Some((byte >> bit_shift_count) as u8);
        }

        for i in T - byte_shift_count..T {
            result.data[i] = msb_fill;
        }

        if bit_shift_count != 0 {
            result.data[T - byte_shift_count - 1] = Some(
                (msb_fill.unwrap() << (8 - bit_shift_count))
                    | result.data[T - byte_shift_count - 1].unwrap(),
            );
        }

        result
    }
}

pub trait ShraAssign<Rhs = Self> {
    fn shra_assign(&mut self, rhs: Rhs);
}

impl<const T: usize, const R: usize> ShraAssign<Bytes<R>> for Bytes<T> {
    fn shra_assign(&mut self, rhs: Bytes<R>) {
        *self = self.shra(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    #[test]
    fn test_known() {
        for _i in 0..u8::MAX {
            for _j in 0..u8::MAX {
                let a = random::<u16>();
                let b = random::<u16>();
                let lhs = Bytes::<2>::from(a);
                let rhs = Bytes::<2>::from(b);

                assert_eq!(lhs + rhs, Bytes::<2>::from(a + b));
                assert_eq!(lhs - rhs, Bytes::<2>::from(a - b));
                assert_eq!(lhs & rhs, Bytes::<2>::from(a & b));
                assert_eq!(lhs | rhs, Bytes::<2>::from(a | b));
                assert_eq!(lhs ^ rhs, Bytes::<2>::from(a ^ b));
                assert_eq!(!lhs, Bytes::<2>::from(!a));
            }
        }
    }

    #[test]
    fn test_known_assign() {
        for _i in 0..u8::MAX {
            for _j in 0..u8::MAX {
                let a = random::<u16>();
                let b = random::<u16>();
                let lhs = Bytes::<2>::from(a);
                let rhs = Bytes::<2>::from(b);

                let mut tmp = lhs;
                tmp += rhs;
                assert_eq!(tmp, Bytes::<2>::from(a + b));

                let mut tmp = lhs;
                tmp -= rhs;
                assert_eq!(tmp, Bytes::<2>::from(a - b));

                let mut tmp = lhs;
                tmp &= rhs;
                assert_eq!(tmp, Bytes::<2>::from(a & b));

                let mut tmp = lhs;
                tmp |= rhs;
                assert_eq!(tmp, Bytes::<2>::from(a | b));

                let mut tmp = lhs;
                tmp ^= rhs;
                assert_eq!(tmp, Bytes::<2>::from(a ^ b));
            }
        }
    }

    #[test]
    fn test_unknown() {
        for i in 0..(u16::MAX as u32) + 1 {
            let lhs = Bytes::<2>::from(i as u16);
            let rhs = Bytes::<2>::unknown();

            assert_eq!(lhs + rhs, Bytes::<2>::unknown());
            assert_eq!(lhs - rhs, Bytes::<2>::unknown());
            if !format!("{}", lhs).contains("00") {
                assert_eq!(lhs & rhs, Bytes::<2>::unknown());
            }
            if !format!("{}", lhs).contains("FF") {
                assert_eq!(lhs | rhs, Bytes::<2>::unknown());
                assert_eq!(lhs ^ rhs, Bytes::<2>::unknown());
            }
            assert_eq!(!rhs, Bytes::<2>::unknown());
        }
    }

    #[test]
    fn test_left_smaller() {
        for _i in 0..u16::MAX {
            let a = random::<u16>();
            let b = random::<u32>();
            let lhs = Bytes::<2>::from(a);
            let rhs = Bytes::<4>::from(b);

            assert_eq!(lhs + rhs, Bytes::<2>::from(a + b as u16));
            assert_eq!(lhs - rhs, Bytes::<2>::from(a - b as u16));
            assert_eq!(lhs & rhs, Bytes::<2>::from(a & b as u16));
            assert_eq!(lhs | rhs, Bytes::<2>::from(a | b as u16));
            assert_eq!(lhs ^ rhs, Bytes::<2>::from(a ^ b as u16));
            assert_eq!(!lhs, Bytes::<2>::from(!a));
        }
    }

    #[test]
    fn test_right_smaller() {
        for _i in 0..u16::MAX {
            let a = random::<u32>();
            let b = random::<u16>();
            let lhs = Bytes::<4>::from(a);
            let rhs = Bytes::<2>::from(b);

            assert_eq!(lhs + rhs, Bytes::<4>::from(a + b as u32));
            assert_eq!(lhs - rhs, Bytes::<4>::from(a - b as u32));
            assert_eq!(lhs & rhs, Bytes::<4>::from(a & b as u32));
            assert_eq!(lhs | rhs, Bytes::<4>::from(a | b as u32));
            assert_eq!(lhs ^ rhs, Bytes::<4>::from(a ^ b as u32));
            assert_eq!(!lhs, Bytes::<4>::from(!a));
        }
    }

    #[test]
    fn test_and_reset() {
        for i in 0..(u16::MAX as u32) + 1 {
            let lhs = Bytes::<2>::from(i as u16);
            let rhs = Bytes::<2>::from(0u16);
            assert_eq!(lhs & rhs, Bytes::<2>::from(0u16));
        }
    }

    #[test]
    fn test_or_overwrite() {
        for i in 0..(u16::MAX as u32) + 1 {
            let lhs = Bytes::<2>::from(i as u16);
            let rhs = Bytes::<2>::from(0xFFFFu16);
            assert_eq!(lhs | rhs, Bytes::<2>::from(0xFFFFu16));
        }
    }

    #[test]
    fn test_xor_flip() {
        for i in 0..(u16::MAX as u32) + 1 {
            let lhs = Bytes::<2>::from(i as u16);
            let rhs = Bytes::<2>::from(0xFFFFu16);
            assert_eq!(lhs ^ rhs, !lhs);
        }
    }

    #[test]
    fn test_shl() {
        for _i in 0..u16::MAX {
            let a = random::<u128>();
            let b = random::<u8>().min(127);
            let lhs = Bytes::<16>::from(a);
            let rhs = Bytes::<1>::from(b);

            assert_eq!(lhs << rhs, Bytes::<16>::from(a << b));
        }
    }

    #[test]
    fn test_shr() {
        for _i in 0..u16::MAX {
            let a = random::<u128>();
            let b = random::<u8>().min(127);
            let lhs = Bytes::<16>::from(a);
            let rhs = Bytes::<1>::from(b);

            assert_eq!(lhs >> rhs, Bytes::<16>::from(a >> b));
        }
    }

    #[test]
    fn test_shra() {
        for _i in 0..u16::MAX {
            let a = random::<u128>();
            let b = random::<u8>().min(127);
            let lhs = Bytes::<16>::from(a);
            let rhs = Bytes::<1>::from(b);

            assert_eq!(lhs.shra(rhs), Bytes::<16>::from(((a as i128) >> b) as u128));
        }
    }

    #[test]
    fn test_from_into() {
        for _i in 0..u8::MAX {
            let a = random::<u8>();
            let b = random::<u16>();
            let c = random::<u32>();
            let d = random::<u64>();
            let e = random::<u128>();

            assert_eq!(Into::<Option<u8>>::into(Bytes::<1>::from(a)), Some(a));
            assert_eq!(Into::<Option<u16>>::into(Bytes::<2>::from(b)), Some(b));
            assert_eq!(Into::<Option<u32>>::into(Bytes::<4>::from(c)), Some(c));
            assert_eq!(Into::<Option<u64>>::into(Bytes::<8>::from(d)), Some(d));
            assert_eq!(Into::<Option<u128>>::into(Bytes::<16>::from(e)), Some(e));
        }
    }

    #[test]
    fn test_from_into_unknown() {
        assert_eq!(Into::<Option<u8>>::into(Bytes::<1>::unknown()), None);
        assert_eq!(Into::<Option<u16>>::into(Bytes::<2>::unknown()), None);
        assert_eq!(Into::<Option<u32>>::into(Bytes::<4>::unknown()), None);
        assert_eq!(Into::<Option<u64>>::into(Bytes::<8>::unknown()), None);
        assert_eq!(Into::<Option<u128>>::into(Bytes::<16>::unknown()), None);
    }

    #[test]
    fn test_index() {
        assert_eq!(Bytes::<1>::unknown()[0], None);
        assert_eq!(Bytes::<1>::from(1u8)[0], Some(1u8));
        assert_eq!(Bytes::<2>::from(0xFF00u16)[1], Some(0xFFu8));
    }

    #[test]
    fn test_cmp() {
        for _i in 0..u16::MAX {
            let a = random::<u32>();
            let b = random::<u32>();
            let lhs = Bytes::<4>::from(a);
            let rhs = Bytes::<4>::from(b);

            assert_eq!(lhs.byte_cmp(rhs), a.cmp(&b));
        }
    }
}
