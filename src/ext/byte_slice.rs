pub trait ByteSliceExt {
    fn read_u16_be(&self, offset: usize) -> u16;
    fn read_u32_be(&self, offset: usize) -> u32;
}

impl ByteSliceExt for [u8] {
    fn read_u16_be(&self, offset: usize) -> u16 {
        u16::from_be_bytes(self[offset..offset + 2].try_into().unwrap())
    }

    fn read_u32_be(&self, offset: usize) -> u32 {
        u32::from_be_bytes(self[offset..offset + 4].try_into().unwrap())
    }
}
