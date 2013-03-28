#[link(name = "snappy",
       vers = "0.1.0",
       uuid = "17d57f36-462f-49c8-a3e1-109a7a4296c8",
       url = "https://github.com/thestinger/rust-snappy")];

#[comment = "snappy bindings"];
#[license = "MIT"];
#[crate_type = "lib"];

extern mod std;

use core::libc::{c_int, size_t};

#[link_args = "-lsnappy"]
extern "C" {
    fn snappy_compress(input: *const u8,
                       input_length: size_t,
                       compressed: *mut u8,
                       compressed_length: *mut size_t) -> c_int;
    fn snappy_uncompress(compressed: *const u8,
                         compressed_length: size_t,
                         uncompressed: *mut u8,
                         uncompressed_length: *mut size_t) -> c_int;
    fn snappy_max_compressed_length(source_length: size_t) -> size_t;
    fn snappy_uncompressed_length(compressed: *const u8,
                                  compressed_length: size_t,
                                  result: *mut size_t) -> c_int;
    fn snappy_validate_compressed_buffer(compressed: *const u8,
                                         compressed_length: size_t) -> c_int;
}

pub fn validate_compressed_buffer(src: &[u8]) -> bool {
    unsafe {
        snappy_validate_compressed_buffer(vec::raw::to_ptr(src),
                                          src.len() as size_t) == 0
    }
}

pub fn compress(src: &[u8]) -> ~[u8] {
    unsafe {
        let srclen = src.len() as size_t;
        let psrc = vec::raw::to_ptr(src);

        let mut dstlen = snappy_max_compressed_length(srclen);
        let mut dst = vec::from_elem(dstlen as uint, 0);
        let pdst = vec::raw::to_mut_ptr(dst);

        let r = snappy_compress(psrc, srclen, pdst, &mut dstlen);
        fail_unless!(r == 0); // SNAPPY_BUFFER_TOO_SMALL should never occur

        dst.truncate(dstlen as uint);
        dst
    }
}

pub fn uncompress(src: &[u8]) -> Option<~[u8]> {
    unsafe {
        let srclen = src.len() as size_t;
        let psrc = vec::raw::to_ptr(src);

        let mut dstlen: size_t = 0;
        snappy_uncompressed_length(psrc, srclen, &mut dstlen);

        let mut dst = vec::from_elem(dstlen as uint, 0);
        let pdst = vec::raw::to_mut_ptr(dst);

        let r = snappy_uncompress(psrc, srclen, pdst, &mut dstlen);

        if r == 0 {
            dst.truncate(dstlen as uint);
            Some(dst)
        } else {
            fail_unless!(r == 1); // SNAPPY_BUFFER_TOO_SMALL should never occur
            None // SNAPPY_INVALID_INPUT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let d = ~[0xde, 0xad, 0xd0, 0x0d];
        let c = compress(d);
        fail_unless!(validate_compressed_buffer(c));
        fail_unless!(uncompress(c) == Some(d));
    }

    #[test]
    fn invalid() {
        let d = ~[0, 0, 0, 0];
        fail_unless!(!validate_compressed_buffer(d));
        fail_unless!(uncompress(d).is_none());
    }
}
