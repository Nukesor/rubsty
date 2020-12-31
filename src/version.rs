pub struct VersionMismatch {
    pub level: VersionLevel,
    pub versions: Vec<RubyVersion>,
}

#[derive(strum_macros::ToString, Debug, Clone)]
pub enum VersionLevel {
    Major,
    Minor,
    Teeny,
    Patch,
}

#[derive(Debug, Clone)]
pub struct RubyVersion {
    pub major: String,
    pub minor: String,
    pub teeny: Option<String>,
    pub patch: Option<String>,
    pub found_in_file: String,
}
impl RubyVersion {
    pub fn print(&self) {
        println!("Detected {} in {}", self, self.found_in_file)
    }

    pub fn from_captures(captures: regex::Captures, filepath: String) -> RubyVersion {
        let major = String::from(captures.name("major").unwrap().as_str());
        let minor = String::from(captures.name("minor").unwrap().as_str());
        let teeny = match captures.name("teeny") {
            Some(teeny) => Some(String::from(teeny.as_str())),
            None => None,
        };
        let patch = match captures.name("patch") {
            Some(patch) => Some(String::from(patch.as_str())),
            None => None,
        };
        RubyVersion {
            major,
            minor,
            teeny,
            patch,
            found_in_file: filepath,
        }
    }
}

impl std::fmt::Display for RubyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.teeny {
            Some(teeny) => match &self.patch {
                Some(patch) => {
                    write!(f, "{}.{}.{}-p{}", self.major, self.minor, teeny, patch)
                }
                None => {
                    write!(f, "{}.{}.{}", self.major, self.minor, teeny)
                }
            },
            None => {
                write!(f, "{}.{}", self.major, self.minor)
            }
        }
    }
}

impl PartialEq for RubyVersion {
    fn eq(&self, other: &Self) -> bool {
        let mut result = self.major == other.major
            && self.minor == other.minor
            && self.found_in_file == other.found_in_file;
        if let Some(teeny) = &self.teeny {
            if let Some(other_teeny) = &other.teeny {
                result = result && teeny == other_teeny
            } else {
                return false;
            }
        }
        if let Some(patch) = &self.patch {
            if let Some(other_patch) = &other.patch {
                result = result && patch == other_patch
            } else {
                return false;
            }
        }
        result
    }
}
impl Eq for RubyVersion {}

impl std::hash::Hash for RubyVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.teeny.hash(state);
        self.patch.hash(state);
        self.found_in_file.hash(state)
    }
}
