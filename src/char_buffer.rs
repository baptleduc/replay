pub struct CharBuffer {
    buf: Vec<u8>,
}

impl CharBuffer {
    pub fn new() -> Self {
        CharBuffer { buf: vec![] }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        CharBuffer { buf }
    }

    /// Return the popped character
    pub fn pop_char(&mut self) -> Option<u8> {
        self.buf.pop()
    }

    pub fn push_char(&mut self, c: u8) {
        self.buf.push(c);
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Return the length of the popped word
    pub fn pop_word(&mut self) -> Option<usize> {
        // Remove trailing spaces
        while self.peek_char() == Some(&b' ') {
            self.pop_char();
        }

        let buf_len = self.buf.len();
        if buf_len == 0 {
            return None; // No word to pop
        }

        for i in (0..buf_len).rev() {
            if self.buf[i] == b' ' {
                let word_start = i + 1;
                self.buf.truncate(i + 1);
                return Some(buf_len - word_start);
            }
        }
        // If no space was found, truncate the buffer to 0
        self.buf.truncate(0);
        Some(buf_len)
    }

    pub fn peek_char(&self) -> Option<&u8> {
        self.buf.last()
    }

    /// Return read-only reference to the buffer
    pub fn get_buf(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for CharBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::CharBuffer;

    #[test]
    fn pop_char_returns_last_byte() {
        let mut buf = CharBuffer::from_vec(vec![b'a', b'b', b'c']);
        assert_eq!(buf.pop_char(), Some(b'c'));
        assert_eq!(buf.pop_char(), Some(b'b'));
        assert_eq!(buf.pop_char(), Some(b'a'));
        assert_eq!(buf.pop_char(), None);
    }

    #[test]
    fn pop_word_truncates_at_last_space() {
        let mut buf = CharBuffer::from_vec(b"hello world test".to_vec());
        assert_eq!(buf.pop_word(), Some(4)); // `test` is 4 long
        assert_eq!(buf.get_buf(), b"hello world "); // With trailing space
        assert_eq!(buf.pop_word(), Some(5)); // `world` is 5 long
        assert_eq!(buf.get_buf(), b"hello "); // With trailing space
        assert_eq!(buf.pop_word(), Some(5)); // `hello` is 5 long
        assert_eq!(buf.get_buf(), b"");
    }

    #[test]
    fn pop_word_empty_buffer() {
        let mut buf = CharBuffer::new();
        assert_eq!(buf.pop_word(), None);
    }

    #[test]
    fn push_char_and_get_buf() {
        let mut buf = CharBuffer::new();
        buf.push_char(b'a');
        buf.push_char(b'b');
        buf.push_char(b'c');
        assert_eq!(buf.get_buf(), b"abc");
    }

    #[test]
    fn clear_resets_buffer() {
        let mut buf = CharBuffer::from_vec(b"something".to_vec());
        buf.clear();
        assert_eq!(buf.get_buf(), b"");
        assert_eq!(buf.pop_char(), None);
    }

    #[test]
    fn peek_char_returns_last_without_removing() {
        let mut buf = CharBuffer::from_vec(vec![b'x', b'y']);
        assert_eq!(buf.peek_char(), Some(&b'y'));
        assert_eq!(buf.get_buf(), b"xy");
        buf.pop_char();
        assert_eq!(buf.peek_char(), Some(&b'x'));
    }

    #[test]
    fn peek_char_empty_buffer() {
        let buf = CharBuffer::new();
        assert_eq!(buf.peek_char(), None);
    }

    #[test]
    fn from_vec_constructor_works() {
        let buf = CharBuffer::from_vec(b"abc".to_vec());
        assert_eq!(buf.get_buf(), b"abc");
    }
}
