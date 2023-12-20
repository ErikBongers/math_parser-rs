//MIT license applies.
//based on https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_span/lib.rs.html#1781

/// Identifies an offset of a multi-byte character in a `SourceFile`.
#[derive(Debug)]
pub struct MultiByteChar {
    pub pos: usize,
    pub bytes: u8,
}
pub struct FileIndex {
    pub lines: Vec<usize>,
    pub multi_byte_chars: Vec<MultiByteChar>,
}
impl FileIndex {
    pub fn new(src: &str) -> Self {
        let mut file_index = FileIndex {lines: vec![], multi_byte_chars: vec![]};
        file_index.index_lines_and_multibytechars(src);
        file_index
    }
    fn index_lines_and_multibytechars(&mut self, src: &str) {
        let scan_len = src.len();
        let mut i = 0;
        let src_bytes = src.as_bytes();

        while i < scan_len {
            let byte = unsafe { *src_bytes.get_unchecked(i) };
            let mut char_len = 1;

            if byte < 32 {
                let pos = i;

                if byte == b'\n' {
                    self.lines.push(pos + 1);
                }
            } else if byte >= 127 {
                let c = src[i..].chars().next().unwrap();
                char_len = c.len_utf8();

                let pos = i;

                if char_len > 1 {
                    let mbc = MultiByteChar { pos, bytes: char_len as u8 };
                    self.multi_byte_chars.push(mbc);
                }
            }
            i += char_len;
        }
    }
    pub fn bytepos_to_charpos(&self, bpos: usize) -> usize {
        // The number of extra bytes due to multibyte chars in the `SourceFile`.
        let mut total_extra_bytes = 0;

        for mbc in self.multi_byte_chars.iter() {
            if mbc.pos < bpos {
                // Every character is at least one byte, so we only
                // count the actual extra bytes.
                total_extra_bytes += mbc.bytes as u32 - 1;
            } else {
                break;
            }
        }
        bpos - total_extra_bytes as usize
    }

    pub fn get_line_and_column(&self, pos: usize) -> (usize, usize) {
        let chpos = self.bytepos_to_charpos(pos);
        if let Some(a) = self.get_line_index(pos) {
            let line = a + 1; // if Some(a), where past the first '\n', thus on line 1
            let linebpos = self.lines[a];
            let linechpos = self.bytepos_to_charpos(linebpos);
            let col = chpos - linechpos;
            return (line, col);
        }
        (0, chpos)
    }
    /// Finds the line containing the given position. The return value is the
    /// index into the `lines` array of this `SourceFile`, not the 1-based line
    /// number. If the source_file is empty or the position is located before the
    /// first line, `None` is returned.
    fn get_line_index(&self, pos: usize) -> Option<usize> {
        self.lines.partition_point(|x| x <= &pos).checked_sub(1)
    }}


