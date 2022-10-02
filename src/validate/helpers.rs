use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use regex::Regex;

use crate::validate::ValidateParserError;

pub fn validate_continuous_assets(paths: &[PathBuf]) -> Result<()> {
    // Checking the assets are a proper series starting at 0 and ending at n-1
    let num_re = Regex::new(r"^(\d+).json$").unwrap();
    let collection_re = Regex::new(r"^collection.json$").unwrap();
    let mut collection_found = false;

    let num_series = paths
        .iter()
        .filter_map(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            if collection_re.is_match(name) {
                collection_found = true;
                return None;
            }
            num_re
                .captures(name)
                .map(|number| number[1].parse::<usize>().unwrap())
        })
        .collect::<Vec<usize>>();

    if collection_found && num_series.len() != paths.len() - 1 {
        return Err(ValidateParserError::UnexpectedFilesFound.into());
    }
    if !collection_found && num_series.len() != paths.len() {
        return Err(ValidateParserError::UnexpectedFilesFound.into());
    }

    if num_series.is_empty() {
        return Err(ValidateParserError::NoAssetsFound.into());
    }

    // Sum of series given we expect:
    // a_0 = 0 , a_n = num_series.size() - 1 , n = num_series.size() => n * (a_0 + a_n) / 2
    // https://en.wikipedia.org/wiki/Arithmetic_progression

    let target_sum = num_series.len() * (num_series.len() - 1) / 2;
    let mut sum: usize = 0;
    let mut redundant: HashSet<usize> = HashSet::new();
    for num in &num_series {
        if redundant.contains(num) {
            return Err(ValidateParserError::RedundantFile(*num).into());
        } else if num >= &num_series.len() {
            return Err(ValidateParserError::FileOutOfRange(*num).into());
        } else {
            redundant.insert(*num);
            sum += num;
        }
    }

    if sum != target_sum {
        return Err(ValidateParserError::NonContinuousSeries.into());
    }

    Ok(())
}

#[test]
fn test_validate_continuous_assets_success() {
    let paths = vec![
        PathBuf::from("assets/0.json"),
        PathBuf::from("assets/1.json"),
        PathBuf::from("assets/2.json"),
        PathBuf::from("assets/3.json"),
        PathBuf::from("assets/4.json"),
    ];
    assert!(validate_continuous_assets(&paths).is_ok());
}

#[test]
fn test_validate_continuous_assets_with_collection_success() {
    let paths = vec![
        PathBuf::from("assets/0.json"),
        PathBuf::from("assets/1.json"),
        PathBuf::from("assets/2.json"),
        PathBuf::from("assets/3.json"),
        PathBuf::from("assets/4.json"),
        PathBuf::from("assets/collection.json"),
    ];
    assert!(validate_continuous_assets(&paths).is_ok());
}

#[test]
fn test_validate_continuous_assets_fail_out_of_range() {
    let paths = vec![
        PathBuf::from("assets/0.json"),
        PathBuf::from("assets/1.json"),
        PathBuf::from("assets/2.json"),
        PathBuf::from("assets/9.json"),
        PathBuf::from("assets/collection.json"),
    ];
    let result = validate_continuous_assets(&paths);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "File 9.json is out of expected range"
    );
}

#[test]
fn test_validate_continuous_assets_fail_redundant_file() {
    let paths = vec![
        PathBuf::from("assets/0.json"),
        PathBuf::from("assets/1.json"),
        PathBuf::from("assets/2.json"),
        PathBuf::from("assets/2.json"),
        PathBuf::from("assets/collection.json"),
    ];
    let result = validate_continuous_assets(&paths);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Redundant file 2.json");
}

#[test]
fn test_validate_continuous_assets_fail_bad_naming() {
    let paths = vec![
        PathBuf::from("assets/0.json"),
        PathBuf::from("assets/xyz1.json"),
        PathBuf::from("assets/-2.json"),
        PathBuf::from("assets/collection.json"),
    ];
    let result = validate_continuous_assets(&paths);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Unexpected files found in assets directory"
    );
}

#[test]
fn test_validate_continuous_assets_fail_no_assets_found_with_collection() {
    let paths = vec![
        PathBuf::from("assets/hello_world.json"),
        PathBuf::from("assets/collection.json"),
    ];
    let result = validate_continuous_assets(&paths);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Unexpected files found in assets directory"
    );
}

#[test]
fn test_validate_continuous_assets_fail_no_assets_found() {
    let paths = vec![PathBuf::from("assets/hello_world.json")];
    let result = validate_continuous_assets(&paths);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Unexpected files found in assets directory"
    );
}
