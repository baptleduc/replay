pub struct CharBuffer {
    buf: Vec<u8>,
}

impl CharBuffer {
    pub fn new() -> Self {
        CharBuffer { buf: vec![] }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        CharBuffer { buf: buf }
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
        let buf_len = self.buf.len();
        for i in (0..buf_len).rev() {
            if self.buf[i] == b' ' {
                self.buf.truncate(i);
                return Some(buf_len - i);
            }
        }

        None
    }

    pub fn peek_char(&self) -> Option<&u8> {
        self.buf.last()
    }

    /// Return read-only reference to the buffer
    pub fn get_buf(&self) -> &[u8] {
        &self.buf
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
        assert_eq!(buf.pop_char(), None); // vide → None
    }

    #[test]
    fn pop_word_truncates_at_last_space() {
        let mut buf = CharBuffer::from_vec(b"hello world test".to_vec());
        assert_eq!(buf.pop_word(), Some(5)); // supprime " test"
        assert_eq!(buf.get_buf(), b"hello world");
        assert_eq!(buf.pop_word(), Some(6)); // supprime " world"
        assert_eq!(buf.get_buf(), b"hello");
        assert_eq!(buf.pop_word(), None); // pas d'espace restant
        assert_eq!(buf.get_buf(), b"hello");
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
        assert_eq!(buf.peek_char(), Some(&b'y')); // peek dernier
        assert_eq!(buf.get_buf(), b"xy"); // pas consommé
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
