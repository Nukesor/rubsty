extern crate itertools;
extern crate regex;

mod version;
mod ruby;

use comfy_table::Cell;
use comfy_table::Table;
use version::*;

fn main() {
    let mismatches = ruby::detect_version_mismatches();
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
