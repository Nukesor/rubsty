use crate::version::{RubyVersion, VersionMismatch, VersionLevel};
use itertools::Itertools;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn detect_version_mismatches() -> Vec<VersionMismatch<'static>> {
    let paths = fs::read_dir("./fixtures/different_versions").unwrap();
    let versions = parse_files_for_versions(paths);
    let mismatches = build_version_mismatches(versions);
    return mismatches;
}

fn build_version_mismatches(versions: Vec<RubyVersion>) -> Vec<VersionMismatch<'static>> {
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
    let mismatches = build_version_mismatches(&versions);
    assert!(mismatches.is_empty());
}

#[test]
fn test_with_different_version() {
    let paths = fs::read_dir("./fixtures/different_versions").unwrap();
    let versions = parse_files_for_versions(paths);
    let mismatches = build_version_mismatches(&versions);
    println!("{}", mismatches.len());
    assert!(mismatches.len() == 1);
}