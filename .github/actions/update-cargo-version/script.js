const fs = require('fs');

const MAJOR = 'major';
const MINOR = 'minor';
const PATCH = 'patch';

/**
 * Compute the updated version based on the semvar value
 *
 * @param {semvar} string The string representation of the version update to make
 * @param {version} string The current semantic version
 * @return {string} The updated version
 */
const getUpdatedVersion = (semvar, version) => {
  let [major, minor, patch] = version.split('.').map((v) => +v);

  if (semvar === MAJOR) {
    major += 1;
  } else if (semvar === MINOR) {
    minor += 1;
  } else if (semvar === PATCH) {
    patch += 1;
  }

  return `${major}.${minor}.${patch}`;
};

/**
 * Parse the Cargo.toml for the current version, and bump the version based on the semvar update value.
 *
 * @param {github} obj An @actions/github object
 * @param {toml} obj A @iarna/toml object
 * @param {cargo_path} string The path to the Cargo.toml to update
 * @param {semvar} string The semvar udpate value
 * @return void
 */
module.exports = ({ core, toml }, cargo_path, semvar) => {
  if ([MAJOR, MINOR, PATCH].includes(semvar)) {
    // Verify read and write permissions
    fs.access(cargo_path, fs.constants.R_OK | fs.constants.W_OK, (err) => {
      console.log('\n> Checking Permission for reading and writing to file');
      if (err) {
        throw new Error('No read and write access');
      }

      let tomlObj = toml.parse(fs.readFileSync(cargo_path, 'utf-8'));
      if (!tomlObj.package) throw new Error('No package tag defined in Cargo.toml');

      tomlObj.package.version = getUpdatedVersion(semvar, tomlObj.package.version);

      fs.writeFileSync(cargo_path, toml.stringify(tomlObj));
      // set output var to read in subsequent steps
      core.exportVariable('UPDATED_VERSION', tomlObj.package.version);
      core.exportVariable('UPDATE_OK', true);
    });
  }

  // set default output vars
  core.exportVariable('UPDATED_VERSION', '0.0.0');
  core.exportVariable('UPDATE_OK', false);
};
