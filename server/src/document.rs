use lsp_types::{Position, Range, Url};

/// State for one open document.
pub struct OpenDocument {
    pub uri: Url,
    pub version: i32,
    pub text: String,
}

/// Converts between byte offsets and LSP positions (UTF-16 columns).
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self { starts }
    }

    pub fn position(&self, text: &str, mut offset: usize) -> Position {
        offset = offset.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let col = text[self.starts[line]..offset].encode_utf16().count();
        Position::new(line as u32, col as u32)
    }

    pub fn range(&self, text: &str, start: usize, end: usize) -> Range {
        Range::new(self.position(text, start), self.position(text, end))
    }

    pub fn offset(&self, text: &str, pos: Position) -> usize {
        let Some(&start) = self.starts.get(pos.line as usize) else {
            return text.len();
        };
        let end = self
            .starts
            .get(pos.line as usize + 1)
            .map(|e| e - 1)
            .unwrap_or(text.len());
        let mut units = 0u32;
        for (i, c) in text[start..end].char_indices() {
            if units >= pos.character {
                return start + i;
            }
            units += c.len_utf16() as u32;
        }
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_round_trip() {
        let text = "a😀b\nñx";
        let idx = LineIndex::new(text);
        // 😀 is 4 bytes, 2 UTF-16 units.
        assert_eq!(idx.position(text, 5), Position::new(0, 3));
        assert_eq!(idx.offset(text, Position::new(0, 3)), 5);
        assert_eq!(idx.position(text, text.len()), Position::new(1, 2));
        assert_eq!(idx.offset(text, Position::new(1, 1)), 9);
    }
}
