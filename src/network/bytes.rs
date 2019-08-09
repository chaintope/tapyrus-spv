use bytes;
use bytes::BufMut;
use std::io::Write;
use std::io::Error;

pub struct BytesMut<'a> {
    inner: &'a mut bytes::BytesMut,
}

impl<'a> BytesMut<'a> {
    pub fn new(b: &'a mut bytes::BytesMut) -> BytesMut {
        BytesMut { inner: b }
    }

    pub fn remaining_mut(&self) -> usize {
        self.inner.remaining_mut()
    }
}

impl<'a> Write for BytesMut<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let remaining_capacity = self.inner.remaining_mut();
        if remaining_capacity >= buf.len() {
            self.inner.put_slice(buf);
            return Ok(buf.len());
        } else {
            self.inner.put_slice(&buf[..remaining_capacity]);
            return Ok(remaining_capacity);
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        panic!("cannot flush ByteMut.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    fn get_random_bytes(len: usize) -> Vec<u8> {
        (0..len).map(|_| { rand::random::<u8>() }).collect()
    }

    #[test]
    fn write_test() {
        // The capacity will be allocated larger than 32 value.
        let mut buf = bytes::BytesMut::with_capacity(10);
        let capacity = buf.capacity();

        let mut bytes_mut = BytesMut::new(&mut buf);

        // write 5 bytes and it can be written.
        assert_eq!(bytes_mut.write(&get_random_bytes(5)).unwrap(), 5);
        assert_eq!(bytes_mut.remaining_mut(), capacity - 5);

        // write bytes over capacity length, just current capacity length is written.
        assert_eq!(bytes_mut.write(&get_random_bytes(capacity)).unwrap(), capacity - 5);
        assert_eq!(bytes_mut.remaining_mut(), 0);

        // write to buffer which has full capacity bytes, no more written.
        assert_eq!(bytes_mut.write(&get_random_bytes(1)).unwrap(), 0);
        assert_eq!(bytes_mut.remaining_mut(), 0);
    }
}