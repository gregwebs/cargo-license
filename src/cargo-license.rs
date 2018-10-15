extern crate ansi_term;
extern crate cargo_license;
extern crate getopts;
extern crate csv;

use std::env;
use ansi_term::Colour::Green;
use std::io;
use getopts::Options;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::btree_map::Entry::*;

fn print_full_licenses(dependencies: Vec<cargo_license::Dependency>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["library", "license", "license-text"]).unwrap();
    for dependency in dependencies {
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        if let Some(license_texts) = dependency.get_license_text() {
            for license_text in license_texts {
                wtr.write_record(&[dependency.name.clone(), license.clone(), license_text]).unwrap();
            }
        } else {
            wtr.write_record(&[dependency.name, license, "N/A".to_owned()]).unwrap();
        }
    }
}

fn group_by_license_type(dependencies: Vec<cargo_license::Dependency>, display_authors: bool) {
    let mut table: BTreeMap<String, Vec<cargo_license::Dependency>> = BTreeMap::new();

    for dependency in dependencies {
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        match table.entry(license) {
            Vacant(e) => {
                e.insert(vec![dependency]);
            }
            Occupied(mut e) => {
                e.get_mut().push(dependency);
            }
        };
    }

    for (license, crates) in table {
        let crate_names = crates.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        if display_authors {
            let crate_authors = crates
                .iter()
                .flat_map(|c| c.get_authors().unwrap_or(vec![]))
                .collect::<BTreeSet<_>>();
            println!(
                "{} ({})\n{}\n{} {}",
                Green.bold().paint(license),
                crates.len(),
                crate_names.join(", "),
                Green.paint("by"),
                crate_authors.into_iter().collect::<Vec<_>>().join(", ")
            );
        } else {
            println!(
                "{} ({}): {}",
                Green.bold().paint(license),
                crates.len(),
                crate_names.join(", ")
            );
        }
    }
}

fn one_license_per_line(dependencies: Vec<cargo_license::Dependency>, display_authors: bool) {
    for dependency in dependencies {
        let name = dependency.name.clone();
        let version = dependency.version.clone();
        let license = dependency.get_license().unwrap_or("N/A".to_owned());
        let source = dependency.source.clone();
        if display_authors {
            let authors = dependency.get_authors().unwrap_or(vec![]);
            println!(
                "{}: {}, \"{}\", {}, {} \"{}\"",
                Green.bold().paint(name),
                version,
                license,
                source,
                Green.paint("by"),
                authors.into_iter().collect::<Vec<_>>().join(", ")
            );
        } else {
            println!(
                "{}: {}, \"{}\", {}",
                Green.bold().paint(name),
                version,
                license,
                source
            );
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let program = args[0].clone();
    opts.optflag("a", "authors", "Display crate authors");
    opts.optflag("d", "do-not-bundle", "Output one license per line.");
    opts.optflag("f", "full", "Display full licenses.");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let display_authors = matches.opt_present("authors");
    let do_not_bundle = matches.opt_present("do-not-bundle");
    let display_full_licenses = matches.opt_present("full");

    let dependencies = match cargo_license::get_dependencies_from_cargo_lock() {
        Ok(m) => m,
        Err(err) => {
            println!(
                "Cargo.lock file not found. Try building the project first.\n{}",
                err
            );
            std::process::exit(1);
        }
    };

    if display_full_licenses {
        print_full_licenses(dependencies);
    } else if do_not_bundle {
        one_license_per_line(dependencies, display_authors);
    } else {
        group_by_license_type(dependencies, display_authors);
    }
}
