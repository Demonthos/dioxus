use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use core::slice;
use std::fmt::{Display, Formatter, Write};
use std::hint::black_box;

const fn min_size(slice: &'static [&'static str]) -> usize {
    let mut idx = 0;
    let mut size = 0;
    while idx < slice.len() {
        let s = slice[idx];
        size += s.len();
        idx += 1;
    }
    size
}

#[derive(Debug, Clone, Copy)]
pub struct DiffableArguments<'a> {
    pub static_segments: &'static [&'static str],
    pub dynamic_segments: &'a [Entry<'a>],
}

impl<'a> DiffableArguments<'a> {
    pub fn to_str(&self) -> Option<&'a str> {
        if let DiffableArguments {
            static_segments: ["", ""],
            dynamic_segments: [Entry::Str(s)],
        } = self
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn to_bump_str(self, bump: &Bump) -> bumpalo::collections::String {
        let mut bump_str =
            bumpalo::collections::String::with_capacity_in(min_size(self.static_segments), bump);
        for (static_seg, dynamic_seg) in self
            .static_segments
            .iter()
            .zip(self.dynamic_segments.iter())
        {
            bump_str.write_str(static_seg).unwrap();
            match dynamic_seg {
                Entry::U64(u) => {
                    u.write(unsafe { bump_str.as_mut_vec() });
                }
                Entry::Usize(u) => {
                    u.write(unsafe { bump_str.as_mut_vec() });
                }
                Entry::I64(i) => {
                    i.write(unsafe { bump_str.as_mut_vec() });
                }
                Entry::F64(f) => bump_str.write_str(f.to_string().as_str()).unwrap(),
                Entry::Bool(b) => match b {
                    true => {
                        bump_str.write_str("true").unwrap();
                    }
                    false => {
                        bump_str.write_str("false").unwrap();
                    }
                },
                Entry::Char(c) => bump_str.write_char(*c).unwrap(),
                Entry::Str(s) => bump_str.write_str(s).unwrap(),
            }
        }
        bump_str
            .write_str(self.static_segments.last().unwrap())
            .unwrap();
        bump_str
    }
}

#[test]
fn displays() {
    let bump = Bump::new();
    for num in 0..10000 {
        let diffable = DiffableArguments {
            static_segments: &["hello ", ", ", " welcome"],
            dynamic_segments: &[
                (&mut &"world").into_entry(&bump),
                (&mut &num).into_entry(&bump),
            ],
        };
        let bump = Bump::new();
        let string = diffable.to_bump_str(&bump);
        assert_eq!(string, format!("hello world, {num} welcome"));
    }
}

impl PartialEq for DiffableArguments<'_> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        if !std::ptr::eq(self.static_segments, other.static_segments) {
            false
        } else {
            for i in 0..self.dynamic_segments.len() {
                if unsafe {
                    self.dynamic_segments.get_unchecked(i)
                        != other.dynamic_segments.get_unchecked(i)
                } {
                    return false;
                }
            }
            true
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Entry<'a> {
    U64(u64),
    Usize(usize),
    I64(i64),
    F64(f64),
    Bool(bool),
    Char(char),
    Str(&'a str),
}

pub trait IntoEntry<'a> {
    fn into_entry(self, bump: &'a Bump) -> Entry<'a>;
}

impl<'a, T: Display> IntoEntry<'a> for &T {
    #[inline(always)]
    fn into_entry(self, bump: &'a Bump) -> Entry<'a> {
        Entry::Str(bumpalo::format!(in bump, "{}", self).into_bump_str())
    }
}

impl<'a> IntoEntry<'a> for &mut &&'a str {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Str(self)
    }
}

impl<'a> IntoEntry<'a> for &mut &u64 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U64(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &u32 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U64(**self as u64)
    }
}

impl<'a> IntoEntry<'a> for &mut &u16 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U64(**self as u64)
    }
}

impl<'a> IntoEntry<'a> for &mut &u8 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U64(**self as u64)
    }
}

impl<'a> IntoEntry<'a> for &mut &usize {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Usize(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &i64 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I64(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &i32 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I64(**self as i64)
    }
}

impl<'a> IntoEntry<'a> for &mut &i16 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I64(**self as i64)
    }
}

impl<'a> IntoEntry<'a> for &mut &i8 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I64(**self as i64)
    }
}

impl<'a> IntoEntry<'a> for &mut &f64 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::F64(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &f32 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::F64(**self as f64)
    }
}

impl<'a> IntoEntry<'a> for &mut &bool {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Bool(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &char {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Char(**self)
    }
}

impl PartialEq for Entry<'_> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::Usize(l0), Self::Usize(r0)) => l0 == r0,
            (Self::I64(l0), Self::I64(r0)) => l0 == r0,
            (Self::F64(l0), Self::F64(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => std::ptr::eq(l0, r0) || l0 == r0,
            _ => false,
        }
    }
}

pub trait Writable {
    fn write(self, into: &mut BumpVec<u8>);
}

macro_rules! write_unsized {
    ($t: ty) => {
        impl Writable for $t {
            #[inline(always)]
            fn write(self, to: &mut BumpVec<u8>) {
                let mut n = self;
                let mut n2 = n;
                let mut num_digits = 0;
                while n2 > 0 {
                    n2 /= 10;
                    num_digits += 1;
                }
                let len = num_digits.max(1);
                to.reserve(len);
                let ptr = to.as_mut_ptr().cast::<u8>();
                let old_len = to.len();
                let mut i = len - 1;
                loop {
                    unsafe { ptr.add(old_len + i).write((n % 10) as u8 + b'0') }
                    n /= 10;

                    if n == 0 {
                        break;
                    } else {
                        i -= 1;
                    }
                }

                #[allow(clippy::uninit_vec)]
                unsafe {
                    to.set_len(old_len + (len - i));
                }
            }
        }
    };
}

macro_rules! write_sized {
    ($t: ty) => {
        impl Writable for $t {
            #[inline(always)]
            fn write(self, to: &mut BumpVec<u8>) {
                let neg = self < 0;
                let mut n = if neg {
                    match self.checked_abs() {
                        Some(n) => n,
                        None => <$t>::MAX / 2 + 1,
                    }
                } else {
                    self
                };
                let mut n2 = n;
                let mut num_digits = 0;
                while n2 > 0 {
                    n2 /= 10;
                    num_digits += 1;
                }
                num_digits = num_digits.max(1);
                let len = if neg { num_digits + 1 } else { num_digits };
                to.reserve(len);
                let ptr = to.as_mut_ptr().cast::<u8>();
                let old_len = to.len();
                let mut i = len - 1;
                loop {
                    unsafe { ptr.add(old_len + i).write((n % 10) as u8 + b'0') }
                    n /= 10;

                    if n == 0 {
                        break;
                    } else {
                        i -= 1;
                    }
                }

                if neg {
                    i -= 1;
                    unsafe { ptr.add(old_len + i).write(b'-') }
                }

                #[allow(clippy::uninit_vec)]
                unsafe {
                    to.set_len(old_len + (len - i));
                }
            }
        }
    };
}

write_unsized!(u8);
write_unsized!(u16);
write_unsized!(u32);
write_unsized!(u64);
write_unsized!(u128);
write_unsized!(usize);

write_sized!(i8);
write_sized!(i16);
write_sized!(i32);
write_sized!(i64);
write_sized!(i128);
write_sized!(isize);
