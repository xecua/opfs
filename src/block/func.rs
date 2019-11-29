use super::types::superblock;
pub fn u8_slice_as_superblock(s: &[u8]) -> superblock {
    let p = s.as_ptr() as *const [u8; std::mem::size_of::<superblock>()];
    unsafe { std::mem::transmute(*p) }
}
