use std::path::PathBuf;

pub static DEFAULT_SRC_CONTENTS: &String = &String::new();

pub struct ScriptSource<'a> {
    pub path: PathBuf,
    pub contents: &'a String,
    pub chars: Vec<char>,
    pub lines: Vec<&'a str>,
}

impl<'a> ScriptSource<'a> {
    pub fn new(path: PathBuf, contents: &'a String) -> ScriptSource<'a> {
        ScriptSource {
            path,
            chars: contents.chars().collect(),
            lines: contents.split("\n").collect(),
            contents,
        }
    }
}

impl Default for ScriptSource<'_> {
    fn default() -> Self {
        ScriptSource::new(PathBuf::new(), DEFAULT_SRC_CONTENTS)
    }
}

impl Clone for ScriptSource<'_> {
    fn clone(&self) -> Self {
        ScriptSource::new(self.path.clone(), self.contents)
    }
}

pub struct MeowindArguments {
    pub path: PathBuf,
}
