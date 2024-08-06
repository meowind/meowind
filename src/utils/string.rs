use unicode_segmentation::UnicodeSegmentation;

pub trait StringUtils {
    fn count(&self) -> usize;
}

impl StringUtils for String {
    fn count(&self) -> usize {
        self.as_str().graphemes(true).count()
    }
}

impl StringUtils for &str {
    fn count(&self) -> usize {
        self.graphemes(true).count()
    }
}
