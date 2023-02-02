use crate::sender::ProtocolEncoder;

pub(crate) struct PulsedataBuffer<const S: usize> {
    pub buf: [u32; S],
    pub offset: usize,
}

impl<const S: usize> PulsedataBuffer<S> {
    pub fn new() -> Self {
        Self {
            buf: [0; S],
            offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn load<SendProto: ProtocolEncoder<F>, const F: u32>(&mut self, c: &SendProto::Cmd) {
        let len = SendProto::encode(c, &mut self.buf[self.offset..]);
        self.offset += len;
    }

    pub fn get(&self, index: usize) -> Option<u32> {
        self.buffer().get(index).cloned()
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buf[..self.offset]
    }
}

impl<'a, const S: usize> IntoIterator for &'a PulsedataBuffer<S> {
    type Item = u32;
    type IntoIter = PulseIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PulseIterator {
            pulses: &self.buf[0..self.offset],
            pos: 0,
        }
    }
}

pub struct PulseIterator<'a> {
    pulses: &'a [u32],
    pos: usize,
}

impl<'a> Iterator for PulseIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.pulses.len() {
            None
        } else {
            let r = self.pulses[self.pos];
            self.pos += 1;
            Some(r)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use super::*;
    use crate::Protocol;

    #[test]
    fn pulsedata_buffer_with_single_value() {
        let mut buffer = PulsedataBuffer::<8>::new();

        let data = vec![1];
        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(1, buffer.buffer().len());

        let returned_val = buffer.get(0);
        assert_eq!(Some(1), returned_val);
    }

    #[test]
    fn pulsedata_buffer_with_multiple_values() {
        let mut buffer = PulsedataBuffer::<8>::new();

        let data = vec![0, 1, 2];
        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(3, buffer.buffer().len());

        for (idx, &val) in data.iter().enumerate() {
            assert_eq!(Some(val), buffer.get(idx))
        }
    }

    #[test]
    fn pulsedata_buffer_with_multiple_writes() {
        let mut buffer = PulsedataBuffer::<8>::new();

        let data = vec![0, 1, 2];

        buffer.load::<StaticEncoder, 0>(&data);
        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(6, buffer.buffer().len());

        for (idx, &val) in data.iter().chain(data.iter()).enumerate() {
            assert_eq!(Some(val), buffer.get(idx))
        }
    }

    #[test]
    fn pulsedata_buffer_get_out_of_bounds() {
        let mut buffer = PulsedataBuffer::<8>::new();

        let data = vec![0];
        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(None, buffer.get(1));
    }

    #[test]
    fn pulsedata_buffer_reset() {
        let mut buffer = PulsedataBuffer::<8>::new();

        let data = vec![0];

        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(Some(0), buffer.get(0));

        buffer.reset();
        assert_eq!(None, buffer.get(0));

        buffer.load::<StaticEncoder, 0>(&data);
        assert_eq!(Some(0), buffer.get(0));
    }

    struct StaticEncoder;

    impl Protocol for StaticEncoder {
        type Cmd = Vec<u32>;
    }

    impl<const FREQ: u32> ProtocolEncoder<FREQ> for StaticEncoder {
        type EncoderData = u32;

        const DATA: Self::EncoderData = 0;

        fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
            for (dst, src) in buf.iter_mut().zip(cmd) {
                *dst = *src;
            }

            usize::min(buf.len(), cmd.len())
        }
    }
}
