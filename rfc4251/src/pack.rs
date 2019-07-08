use num_bigint::BigInt;
use num_traits::Zero;

#[derive(Default)]
pub struct Packer(Vec<u8>);

impl Packer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn inner(self) -> Vec<u8> {
        self.0
    }

    pub fn pack<T>(&mut self, value: T)
    where
        T: Pack,
    {
        value.pack(self)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes)
    }
}

pub trait Pack: Sized {
    fn pack(self, packer: &mut Packer);
}

macro_rules! impl_uint {
    ($t:ty) => {
        impl Pack for $t {
            fn pack(self, packer: &mut Packer) {
                let data = self.to_be_bytes();
                packer.write_bytes(&data);
            }
        }
    };
}

impl_uint!(u8);
impl_uint!(u32);
impl_uint!(u64);

impl Pack for bool {
    fn pack(self, packer: &mut Packer) {
        if self {
            packer.pack(1_u8);
        } else {
            packer.pack(0_u8);
        }
    }
}

impl Pack for &[u8] {
    fn pack(self, packer: &mut Packer) {
        packer.pack(self.len() as u32);
        packer.write_bytes(self);
    }
}

impl Pack for &str {
    fn pack(self, packer: &mut Packer) {
        packer.pack(self.as_bytes());
    }
}

impl Pack for &BigInt {
    fn pack(self, packer: &mut Packer) {
        if self.is_zero() {
            packer.pack(&[] as &[u8]);
        } else {
            packer.pack(&self.to_signed_bytes_be() as &[_]);
        }
    }
}

#[cfg(test)]
mod tests;
