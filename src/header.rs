use ring::digest::{Context, Digest, SHA256};
use uuid::Uuid;

const GUID_SIZE: usize = std::mem::size_of::<uuid::Bytes>();
const HASH_SIZE: usize = 32;

pub const HEADER_SIZE: usize = GUID_SIZE + HASH_SIZE;
pub type Header = [u8; HEADER_SIZE];
pub const CLEAR_HEADER: Header = [0; HEADER_SIZE];


pub fn new() -> Header {
    let guid = Uuid::new_v4().into_bytes();
    let guid_hash = calculate_hash(guid.as_slice());

    let mut header: Header = CLEAR_HEADER;

    let guid_dst = &mut header.as_mut_slice()[..GUID_SIZE];
    guid_dst.copy_from_slice(guid.as_slice());

    let guid_hash_dst = &mut header.as_mut_slice()[GUID_SIZE..GUID_SIZE+HASH_SIZE];
    guid_hash_dst.copy_from_slice(guid_hash.as_ref());

    header
}

pub fn check_integrity(header: &Header) -> bool {
    let guid: &[u8] = &header[..GUID_SIZE];
    let guid_hash = &header[GUID_SIZE..GUID_SIZE+HASH_SIZE];
    calculate_hash(guid).as_ref() == guid_hash
}

fn calculate_hash(guid: &[u8]) -> Digest {
    let mut context = Context::new(&SHA256);
    context.update(guid);
    context.finish()
}