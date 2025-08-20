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

    /// Count the number of trailing spaces at the end of the buffer
    fn count_trailing_spaces(&self) -> usize {
        self.buf.iter().rev().take_while(|&&b| b == b' ').count()
    }

    /// Peek at the last word in the buffer without removing it
    pub fn peek_word(&self) -> Option<&[u8]> {
        let trailing_spaces = self.count_trailing_spaces();
        if self.buf.len() <= trailing_spaces {
            return None;
        }

        // Find the start of the last word
        let mut start = self.buf.len() - trailing_spaces;
        while start > 0 && self.buf[start - 1] != b' ' {
            start -= 1;
        }
        Some(&self.buf[start..self.buf.len() - trailing_spaces])
    }

    /// Remove and return the last word in Vec<u8> as it not longer exist in buffer
    pub fn pop_word(&mut self) -> Option<Vec<u8>> {
        // Return None if buffer is empty or contains only spaces
        let word_slice = self.peek_word()?;

        // Copy the word into a new Vec
        let word = word_slice.to_vec();

        // Count trailing spaces at the end of the buffer
        let trailing_spaces = self.count_trailing_spaces();

        // Truncate the buffer to remove the last word + its trailing spaces
        let new_len = self.buf.len().saturating_sub(word.len() + trailing_spaces);
        self.buf.truncate(new_len);

        Some(word)
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
    fn test_count_trailing_spaces() {
        let mut buf = CharBuffer::from_vec(b"hello world   ".to_vec());
        assert_eq!(buf.count_trailing_spaces(), 3);

        buf.clear();
        assert_eq!(buf.count_trailing_spaces(), 0);

        buf = CharBuffer::from_vec(b"   ".to_vec());
        assert_eq!(buf.count_trailing_spaces(), 3);
    }

    #[test]
    fn pop_char_returns_last_byte() {
        let mut buf = CharBuffer::from_vec(vec![b'a', b'b', b'c']);
        assert_eq!(buf.pop_char(), Some(b'c'));
        assert_eq!(buf.pop_char(), Some(b'b'));
        assert_eq!(buf.pop_char(), Some(b'a'));
        assert_eq!(buf.pop_char(), None);
    }

    #[test]
    fn peek_word() {
        let buf = CharBuffer::from_vec(b"test ".to_vec());
        // `test` is 4 bytes, should not remove anything
        assert_eq!(buf.peek_word(), Some(&b"test"[..]));
        assert_eq!(buf.get_buf(), b"test ");
    }

    #[test]
    fn pop_word_truncates_at_last_space() {
        let mut buf = CharBuffer::from_vec(b"hello world test".to_vec());
        assert_eq!(buf.pop_word(), Some(b"test".to_vec())); // `test` is 4 long
        assert_eq!(buf.get_buf(), b"hello world "); // With trailing space
        assert_eq!(buf.pop_word(), Some(b"world".to_vec())); // `world` is 5 long
        assert_eq!(buf.get_buf(), b"hello "); // With trailing space
        assert_eq!(buf.pop_word(), Some(b"hello".to_vec())); // `hello` is 5 long
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

    #[test]
    fn peek_and_pop_word() {
        let mut buf = CharBuffer::from_vec(b"hello world test".to_vec());
        // `test` is 4 bytes, should not remove anything
        assert_eq!(buf.peek_word(), Some(&b"test"[..]));
        assert_eq!(buf.get_buf(), b"hello world test");

        // pop_word should still work after peeking
        assert_eq!(buf.pop_word(), Some(b"test".to_vec()));
        assert_eq!(buf.get_buf(), b"hello world ");

        // peek on buffer with trailing space
        assert_eq!(buf.peek_word(), Some(&b"world"[..]));
        assert_eq!(buf.get_buf(), b"hello world ");

        // peek and pop on empty buffer
        let mut buf3 = CharBuffer::new();
        assert_eq!(buf3.peek_word(), None);
        assert_eq!(buf3.pop_word(), None);

        // peek and pop on buffer with only spaces
        let mut buf4 = CharBuffer::from_vec(b"     ".to_vec());
        assert_eq!(buf4.peek_word(), None);
        assert_eq!(buf4.pop_word(), None);
    }
}
