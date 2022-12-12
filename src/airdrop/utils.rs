use std::path::Path;

use anyhow::Result;

use crate::{
    airdrop::{
        errors::AirDropError,
        structs::{AirDropResults, AirDropTargets},
    },
    common::*,
};

pub fn write_airdrop_results(airdrop_results: &AirDropResults) -> Result<()> {
    let airdrop_results_path = Path::new("airdrop_results.json");
    let f = File::create(airdrop_results_path)?;
    serde_json::to_writer_pretty(f, airdrop_results)?;
    Ok(())
}

pub fn load_airdrop_results(airdrop_list: &mut AirDropTargets) -> Result<AirDropResults> {
    // Will load previous airdrop results from file and will also sync the results with the targets
    let airdrop_result_path_name = "airdrop_results.json";
    let airdrop_results_path = Path::new(airdrop_result_path_name);
    if !airdrop_results_path.exists() {
        return Ok(AirDropResults::new());
    }

    let file = File::open(airdrop_results_path).map_err(|err| {
        AirDropError::FailedToOpenAirDropResultsFile(
            airdrop_result_path_name.to_string(),
            err.to_string(),
        )
    })?;

    let results: AirDropResults = serde_json::from_reader(file).map_err(|err| {
        AirDropError::AirDropResultsFileWrongFormat(
            airdrop_result_path_name.to_string(),
            err.to_string(),
        )
    })?;

    for (address, transactions) in results.iter() {
        if !airdrop_list.contains_key(address) {
            continue;
        }

        let mut target = *airdrop_list.get(address).unwrap();
        for transaction in transactions.iter() {
            if transaction.status {
                target = target.checked_sub(1).ok_or_else(|| {
                    AirDropError::OverflowDuringSyncOfResultsAndTargetsForAddress(
                        address.to_string(),
                    )
                })?;
            }
        }
        airdrop_list.insert(*address, target);
    }

    Ok(results)
}

pub fn load_airdrop_list(airdrop_list: String) -> Result<AirDropTargets> {
    let airdrop_list_path = Path::new(&airdrop_list);
    if !airdrop_list_path.exists() {
        return Err(AirDropError::AirDropListFileNotFound(airdrop_list).into());
    }

    let file = File::open(airdrop_list_path).map_err(|err| {
        AirDropError::FailedToOpenAirDropListFile(airdrop_list.clone(), err.to_string())
    })?;

    let targets: AirDropTargets = match serde_json::from_reader(file).map_err(|err| {
        AirDropError::AirDropListFileWrongFormat(airdrop_list.clone(), err.to_string())
    }) {
        Ok(targets) => targets,
        Err(err) => return Err(err.into()),
    };

    Ok(targets)
}
