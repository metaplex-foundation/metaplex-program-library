const fs = require('fs');
const path = require('path');
const toml = require('@iarna/toml');
const axios = require('axios');

const { execSync } = require('child_process');

const wrappedExec = (cmd, cwd) => {
  let args = { stdio: 'inherit' };

  if (cwd) {
    args['cwd'] = cwd;
  } else {
    // default to curernt dir
    args['cwd'] = path.join(__dirname);
  }

  execSync(cmd, args);
};

const isPackageType = (actual, target) => actual === target;
// additional equality checks can match other subdirs, e.g. `rust|test|cli|<etc>`
const isCratesPackage = (actual) => isPackageType(actual, 'program');
const isNpmPackage = (actual) => isPackageType(actual, 'js');

// assume most basic versioning for now: `major.minor.patch`; only publish package if local != remote, assuming local >= remote
const shouldPublishPackage = (localVersion, remoteVersion) => localVersion !== remoteVersion;

const addTag = (tagName) => {
  console.log(
    `adding tag ${tagName}`,
    wrappedExec(`git tag ${tagName} && git push origin ${tagName}`),
  );
};

// npm package helpers

const getLocalNpmPackageInfo = (cwd) => {
  const packageJsonPath = `${cwd}/package.json`;
  console.log('reading configuration from ', packageJsonPath);
  var json = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

  return [json.name, json.version];
};

const getLatestPublishedNpmVersion = async (packageName) => {
  try {
    const result = (await axios.get(`https://registry.npmjs.org/${packageName}/latest`)).data;
    return result['version'];
  } catch (e) {
    throw new Error('No configuration found for package: ', packageName);
  }
};

const formatNpmTag = (name, version) => `${name}@${version}`;

const tryPublishNpmPackage = async (npmToken, cwdArgs) => {
  console.log('updating npm package');
  const currentDir = cwdArgs.join('/');

  const [packageName, localPackageVersion] = getLocalNpmPackageInfo(currentDir);
  console.log(`[LOCAL] name: ${packageName}, version: ${localPackageVersion}`);
  const remotePackageVersion = await getLatestPublishedNpmVersion(packageName);
  console.log(`[REMOTE] name: ${packageName}, version: ${remotePackageVersion}`);

  if (shouldPublishPackage(localPackageVersion, remotePackageVersion)) {
    wrappedExec(`echo "//registry.npmjs.org/:_authToken=${npmToken}" > ~/.npmrc`, currentDir);
    wrappedExec(`npm publish`, currentDir);

    addTag(formatNpmTag(packageName, localPackageVersion));
  } else {
    console.log('no publish needed');
  }
};

// crates package helpers

const getLocalCrateInfo = (cwd) => {
  const cargoPath = `${cwd}/Cargo.toml`;
  let tomlObj = toml.parse(fs.readFileSync(cargoPath, 'utf-8'));
  if (!tomlObj.package) throw new Error('No package tag defined in Cargo.toml');

  return [tomlObj.package.name, tomlObj.package.version];
};

const getLatestPublishedCrateVersion = async (crateName) => {
  const result = (await axios.get(`https://crates.io/api/v1/crates/${crateName}/versions`)).data;
  const versions = result['versions'];
  if (versions.length === 0) {
    throw new Error('No versions found for package. Is it published yet? ', crateName);
  }

  // packages are sorted in reverse chronological order
  const latestVersion = versions[0];
  console.log(`Found the following latest version info for crate ${crateName}`, latestVersion);
  if (!versions[0]['num']) {
    throw new Error('Could find version info for package: ', crateName);
  }

  return versions[0]['num'];
};

const formatCrateTag = (name, version) => `${name}-prog-v${version}`;

const tryPublishCratesPackage = async (cargoToken, cwdArgs) => {
  console.log('updating rust package');
  const currentDir = cwdArgs.join('/');

  const [crateName, localCrateVersion] = getLocalCrateInfo(currentDir);
  console.log(`[LOCAL] name: ${crateName}, version: ${localCrateVersion}`);
  const remoteCrateVersion = await getLatestPublishedCrateVersion(crateName);
  console.log(`[REMOTE] name: ${crateName}, version: ${remoteCrateVersion}`);

  // only publish if local != remote crate version
  if (shouldPublishPackage(localCrateVersion, remoteCrateVersion)) {
    wrappedExec(`cargo publish --token ${cargoToken} -p ${crateName}`, currentDir);

    addTag(formatCrateTag(crateName, localCrateVersion));
  } else {
    console.log('no publish needed');
  }
};

/**
 * Iterate through all input packages publish via the respective package managers.
 *
 * @param {packages} arr List of packages to process in the form <pkg-name>/<sub-dir>
 * @param {cargoToken} str token needed to publish Rust crates
 * @param {npmToken} str token needed to publish NPM packages
 * @return void
 */
module.exports = async (packages, cargoToken, npmToken) => {
  if (packages.length === 0) {
    console.log('No packges to publish. Exiting early.');
    return;
  }

  const base = process.env.GITHUB_ACTION_PATH; // alt: path.join(__dirname);
  const splitBase = base.split('/');
  const parentDirsToHome = 4; // ~/<home>/./.github/actions/<name>
  const cwdArgs = splitBase.slice(0, splitBase.length - parentDirsToHome);

  // it's possible exclude is a stringified arr
  const packageIter = typeof packages === 'string' ? (packages = JSON.parse(packages)) : packages;

  for (let package of packageIter) {
    // make sure package doesn't have extra quotes or spacing
    package = package.replace(/\s+|\"|\'/g, '');
    const [name, type] = package.split('/');
    console.log(`Processing package [${name}] of type [${type}]`);
    cwdArgs.push(...[name, type]);
    console.log(`cwdArgs with new package: `, cwdArgs);

    try {
      if (isCratesPackage(type)) await tryPublishCratesPackage(cargoToken, cwdArgs, toml);
      else if (isNpmPackage(type)) await tryPublishNpmPackage(npmToken, cwdArgs);
      else continue;
    } catch (e) {
      console.log(`could not process ${name}/${type} - got error ${e}`);
    } finally {
      // chdir back two levels - back to root, should match original cwd
      cwdArgs.pop();
      cwdArgs.pop();
    }
  }
};
