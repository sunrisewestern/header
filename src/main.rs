use clap::{App, Arg};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process;

fn main() {
    let matches = App::new("header")
        .version("1.0")
        .author("Sunrisewestern")
        .about("Prints the first line of a file, optionally with field numbers.")
        .arg(
            Arg::with_name("delimiter")
                .short('d')
                .long("delimiter")
                .value_name("DELIMITER")
                .help("Sets the field delimiter")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("number")
                .short('n')
                .long("number")
                .help("Show the field number"),
        )
        .arg(
            Arg::with_name("input")
                .value_name("FILE")
                .help("Input file to process")
                .required(true)
                .index(1),
        )
        .get_matches();

    let delimiter = matches.value_of("delimiter").unwrap();
    let show_number = matches.is_present("number");
    let input_file = matches.value_of("input").unwrap();

    let file = File::open(input_file).unwrap_or_else(|err| {
        eprintln!("Error opening file: {}", err);
        process::exit(1);
    });

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    if let Some(Ok(line)) = lines.next() {
        let fields: Vec<&str> = line.split(delimiter).collect();
        for (index, field) in fields.iter().enumerate() {
            let output = if show_number {
                format!("{}\t{}\n", field, index + 1)
            } else {
                format!("{}\n", field)
            };

            if let Err(e) = io::stdout().write_all(output.as_bytes()) {
                if e.kind() == io::ErrorKind::BrokenPipe {
                    // Do not print message to stderr
                    process::exit(0);
                } else {
                    eprintln!("Error writing to stdout: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_main() {
        let test_input = "field1\tfield2\tfield3\n";
        let mut test_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(test_file, "{}", test_input).expect("Failed to write to temp file");

        let test_file_path = test_file.path().to_str().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("-d")
            .arg("\t")
            .arg("-n")
            .arg(test_file_path)
            .output()
            .expect("Failed to execute process");

        assert!(output.status.success());
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output_str.trim(), "field1\t1\nfield2\t2\nfield3\t3");
    }

    #[test]
    fn test_empty_file() {
        let mut test_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_file_path = test_file.path().to_str().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg(test_file_path)
            .output()
            .expect("Failed to execute process");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
    }

    #[test]
    fn test_broken_pipe() {
        let test_input = "field1\tfield2\tfield3\n";
        let mut test_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(test_file, "{}", test_input).expect("Failed to write to temp file");

        let test_file_path = test_file.path().to_str().unwrap();

        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("-d")
            .arg("\t")
            .arg(test_file_path)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let head_output = std::process::Command::new("head")
            .arg("-n")
            .arg("1")
            .stdin(output.stdout.unwrap())
            .output()
            .expect("failed to execute head");

        assert!(head_output.status.success());
        let expected_output = "field1\n".to_string();
        assert_eq!(
            String::from_utf8_lossy(&head_output.stdout),
            expected_output
        );
    }
}
