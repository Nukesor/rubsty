extern crate regex;
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
        match &self.teeny {
            Some(teeny) => {
                match &self.patch {
                    Some(patch) => {
                        println!(
                            "Detected {}.{}.{}-p{} in {}",
                            self.major, self.minor, teeny, patch, self.found_in_file,
                        );
                    }
                    None => {
                        println!(
                            "Detected {}.{}.{} in {}", self.major, self.minor, teeny, self.found_in_file,
                        );
                    }
                }
            },
            None => {
                println!(
                    "Detected {}.{} in {}", self.major, self.minor, self.found_in_file,
                );
            }
        }
    }

    fn from_captures(captures: regex::Captures, filepath: String) -> RubyVersion {
        let major = String::from(captures.name("major").unwrap().as_str());
        let minor = String::from(captures.name("minor").unwrap().as_str());
        let teeny = match captures.name("teeny") {
            Some(teeny) => Some(String::from(teeny.as_str())),
            None => None
        };
        let patch = match captures.name("patch") {
            Some(patch) => Some(String::from(patch.as_str())),
            None => None
        };
        RubyVersion { major, minor, teeny, patch, found_in_file: filepath }
    }
}

fn main() {
    let paths = fs::read_dir("./fixtures/same_versions").unwrap();
    let version = parse_files_for_versions(paths);
}

fn parse_files_for_versions(paths: fs::ReadDir) -> Vec<RubyVersion> {
    let mut versions = Vec::new();
    for path in paths {
        let path = path.unwrap();
        let filename = path.file_name().
            into_string().unwrap();
        let filepath = path.path().display().to_string();
        // let detected_versions = Vec<RubyVersion>
        match filename.as_str() {
            ".ruby-version" => {
                println!("Found .ruby-version");
                let version = process_ruby_version_file(filepath);
                versions.push(version)
            },
            ".tool-versions" => {
                println!("Found .tool-versions");
                match process_tool_versions_file(filepath) {
                    Some(version) => {
                        versions.push(version)
                    },
                    None => {
                        println!("No ruby version defined in .tool-versions")
                    }
                }
            },
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
    let version_regex = Regex::new(
        r"^ruby (?P<major>\d+)\.(?P<minor>\d+)\.(?P<teeny>\d+)(-p(?P<patch>\d+))?"
    ).unwrap();
    let captures = version_regex.captures(&line);
    if let Some(captures) = captures {
        let version = RubyVersion::from_captures(captures, filepath.clone());
        version.print();
        return Some(version)
    }
    None
}

fn process_ruby_version_file(filepath: String) -> RubyVersion {
    let version_regex = Regex::new(
        r"^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<teeny>\d+)(-p(?P<patch>\d+))?"
    ).unwrap();
    let file_content = fs::read_to_string(&filepath).unwrap();
    // println!("{}", file_content);
    let captures = version_regex.captures(&file_content).unwrap();

    let version = RubyVersion::from_captures(captures, filepath.clone());
    version.print();
    version
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[test]
fn test_with_same_versions() {
    let paths = fs::read_dir("./fixtures/same_versions").unwrap();
    parse_files_for_versions(paths);
}

#[test]
fn test_with_different_version() {
    let paths = fs::read_dir("./fixtures/different_versions").unwrap();
    parse_files_for_versions(paths);
}
