extern crate itertools;
extern crate regex;
use comfy_table::Cell;
use comfy_table::Table;
use itertools::Itertools;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

struct RubyVersion {
    major: String,
    minor: String,
    teeny: Option<String>,
    patch: Option<String>,
    found_in_file: String,
}
impl RubyVersion {
    fn print(&self) {
        println!("Detected {} in {}", self, self.found_in_file)
    }

    fn from_captures(captures: regex::Captures, filepath: String) -> RubyVersion {
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

struct VersionMismatch<'a> {
    level: VersionLevel,
    versions: Vec<&'a RubyVersion>,
}

#[derive(strum_macros::ToString, Debug)]
enum VersionLevel {
    Major,
    Minor,
    Teeny,
    Patch,
}

fn main() {
    let paths = fs::read_dir("./fixtures/different_versions").unwrap();
    let versions = parse_files_for_versions(paths);
    let mismatches = detect_version_mismatches(&versions);
    print_mismatches(mismatches);
}

fn print_mismatches(mismatches: Vec<VersionMismatch>) {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    for mismatch in mismatches {
        let mut versions = Vec::new();
        let mut locations = Vec::new();
        for version in mismatch.versions {
            versions.push(format!("{}", version));
            locations.push(format!("{}", version.found_in_file));
        }
        table
            .add_row(vec![Cell::new(mismatch.level.to_string())
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(comfy_table::Color::Red)])
            .add_row(versions)
            .add_row(locations);
    }
    println!("\n{}", table);
}

fn detect_version_mismatches(versions: &Vec<RubyVersion>) -> Vec<VersionMismatch> {
    let mut mismatches = Vec::new();
    for pair in versions.iter().combinations(2) {
        if let Some(mismatch) = compare_two_versions(pair[0], pair[1]) {
            mismatches.push(mismatch);
        }
    }
    mismatches
}

fn compare_two_versions<'a>(
    left_version: &'a RubyVersion,
    right_version: &'a RubyVersion,
) -> Option<VersionMismatch<'a>> {
    if left_version.major != right_version.major {
        let mismatch = VersionMismatch {
            level: VersionLevel::Major,
            versions: vec![left_version, right_version],
        };
        Some(mismatch)
    } else if left_version.minor != right_version.minor {
        let mismatch = VersionMismatch {
            level: VersionLevel::Minor,
            versions: vec![left_version, right_version],
        };
        Some(mismatch)
    } else if left_version.teeny != right_version.teeny {
        let mismatch = VersionMismatch {
            level: VersionLevel::Teeny,
            versions: vec![left_version, right_version],
        };
        Some(mismatch)
    } else if left_version.patch != right_version.patch {
        let mismatch = VersionMismatch {
            level: VersionLevel::Patch,
            versions: vec![left_version, right_version],
        };
        Some(mismatch)
    } else {
        None
    }
}

fn parse_files_for_versions(paths: fs::ReadDir) -> Vec<RubyVersion> {
    let mut versions = Vec::new();
    for path in paths {
        let path = path.unwrap();
        let filename = path.file_name().into_string().unwrap();
        let filepath = path.path().display().to_string();
        // let detected_versions = Vec<RubyVersion>
        match filename.as_str() {
            ".ruby-version" => {
                println!("Found .ruby-version");
                let version = process_ruby_version_file(filepath);
                versions.push(version)
            }
            ".tool-versions" => {
                println!("Found .tool-versions");
                match process_tool_versions_file(filepath) {
                    Some(version) => versions.push(version),
                    None => {
                        println!("No ruby version defined in .tool-versions")
                    }
                }
            }
            _ => println!("Skipping {}", filepath),
        }
    }
    versions
}

fn process_tool_versions_file(filepath: String) -> Option<RubyVersion> {
    if let Ok(lines) = read_lines(&filepath) {
        for line in lines {
            if let Ok(line) = line {
                println!("{}", line);
                if let Some(version) = process_tool_versions_line(line, &filepath) {
                    return Some(version);
                }
            }
        }
    }
    None
}

fn process_tool_versions_line(line: String, filepath: &String) -> Option<RubyVersion> {
    let version_regex =
        Regex::new(r"^ruby (?P<major>\d+)\.(?P<minor>\d+)\.(?P<teeny>\d+)(-p(?P<patch>\d+))?")
            .unwrap();
    let captures = version_regex.captures(&line);
    if let Some(captures) = captures {
        let version = RubyVersion::from_captures(captures, filepath.clone());
        version.print();
        return Some(version);
    }
    None
}

fn process_ruby_version_file(filepath: String) -> RubyVersion {
    let version_regex =
        Regex::new(r"^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<teeny>\d+)(-p(?P<patch>\d+))?").unwrap();
    let file_content = fs::read_to_string(&filepath).unwrap();
    // println!("{}", file_content);
    let captures = version_regex.captures(&file_content).unwrap();

    let version = RubyVersion::from_captures(captures, filepath.clone());
    version.print();
    version
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[test]
fn test_with_same_versions() {
    let paths = fs::read_dir("./fixtures/same_versions").unwrap();
    let versions = parse_files_for_versions(paths);
    let mismatches = detect_version_mismatches(&versions);
    assert!(mismatches.is_empty());
}

#[test]
fn test_with_different_version() {
    let paths = fs::read_dir("./fixtures/different_versions").unwrap();
    let versions = parse_files_for_versions(paths);
    let mismatches = detect_version_mismatches(&versions);
    println!("{}", mismatches.len());
    assert!(mismatches.len() == 1);
}
