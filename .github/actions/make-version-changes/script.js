const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const toml = require('@iarna/toml');

const wrappedExec = (cmd, cwd) => {
  let args = {
    stdio: 'inherit',
  };

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

const parseVersioningCommand = (cmd) => cmd.split(':');
const shouldUpdate = (actual, target) => target === '*' || target === actual;

/**
 * Instead of manually modifying the IDL, regenerate the package library via solita.
 * We assume caching the anchor dependency is done or is feasible. Otherwise, this action
 * can take an increasingly long time to run.
 *
 * @param {*} rootDir - components of MPL repo root dir as an array
 * @param {*} pkg - name of package as a string
 * @returns true if IDL exists, false otherwise
 */
const generatePackageLib = (cwdArgs, pkg) => {
  const rootDir = cwdArgs.slice(0, cwdArgs.length - 2);
  const packageLibDir = [...rootDir, pkg, 'js'].join('/');
  if (!fs.existsSync(packageLibDir)) return;

  // install js dependencies for package
  wrappedExec(`YARN_ENABLE_IMMUTABLE_INSTALLS=false yarn install`, packageLibDir);

  // generate lib for package
  wrappedExec(`yarn api:gen`, packageLibDir);

  // No longer needed due to root yarn.lock not existing.
  // // restore root yarn.lock after installs
  // wrappedExec(`git restore ../../yarn.lock`, packageLibDir);

  // add any changes to the lib dir after generation
  wrappedExec(`git add ${packageLibDir} && git commit --amend -C HEAD`);
};

const updateCratesPackage = async (io, cwdArgs, pkg, semvar) => {
  console.log('updating rust package');
  const currentDir = cwdArgs.join('/');

  // adds git info automatically
  wrappedExec(
    `cargo release --no-publish --no-push --no-confirm --verbose --execute --no-verify --no-tag --config ../../release.toml ${semvar}`,
    currentDir,
  );

  // No longer needed due to root yarn.lock not existing.

  // const rootDir = cwdArgs.slice(0, cwdArgs.length - 2);
  // // if we globally installed `@iarna/toml`, the root `yarn.lock` file will have been committed
  // // along with `cargo release` command. so, we need to resolve this.
  // const rootYarnLockPath = [...rootDir, 'yarn.lock'].join('/');
  // wrappedExec(`git restore --source=HEAD^ --staged -- ${rootYarnLockPath}`);
  // wrappedExec('git commit --amend --allow-empty -C HEAD');

  const crateInfo = getCrateInfo(currentDir);
  console.log(
    `Generating client lib for crate: ${crateInfo.name} at version = ${crateInfo.version}`,
  );

  generatePackageLib(cwdArgs, pkg);

  // finally, push changes from local to remote
  wrappedExec('git push');
};

const updateNpmPackage = (cwdArgs, _pkg, semvar) => {
  console.log(
    'updating js package',
    wrappedExec(`yarn install && npm version ${semvar} && git push`, cwdArgs.join('/')),
  );
};

/**
 * Iterate through all input packages and version commands and process version updates. NPM
 * changes will use `npm version <semvar>` commands. Crates changes will use the cargo release
 * crate to update a crate version. After each update is committed, it will be appended to the
 * branch that invoked this action.
 *
 * @param {github} obj An @actions/github object
 * @param {context} obj An @actions/context object
 * @param {core} obj An @actions/core object
 * @param {glob} obj An @actions/glob object
 * @param {io} obj An @actions/io object
 * @param {packages} arr List of packages to process in the form <pkg-name>/<sub-dir>
 * @param {versioning} arr List of version commands in the form semvar:pkg:type where type = `program|js`
 * @return void
 *
 */
module.exports = async ({ github, context, core, glob, io }, packages, versioning) => {
  const base = process.env.GITHUB_ACTION_PATH; // alt: path.join(__dirname);
  const splitBase = base.split('/');
  const parentDirsToHome = 4; // ~/<home>/./.github/actions/<name>
  const cwdArgs = splitBase.slice(0, splitBase.length - parentDirsToHome);

  if (versioning.length === 0) {
    console.log('No versioning updates to make. Exiting early.');
    return;
  }

  // setup git user config
  wrappedExec('git config user.name github-actions[bot]');
  wrappedExec('git config user.email github-actions[bot]@users.noreply.github.com');

  // versioning = [semvar:pkg:type]
  for (const version of versioning) {
    const [semvar, targetPkg, targetType] = parseVersioningCommand(version);
    if (semvar === 'none') {
      console.log('No versioning updates to make when semvar === none. Continuing.');
      continue;
    }

    for (let package of packages) {
      // make sure package doesn't have extra quotes or spacing
      package = package.replace(/\s+|\"|\'/g, '');

      if (!shouldUpdate(package, targetPkg)) {
        console.log(`No updates for package ${package} based on version command ${version}`);
        continue;
      }

      const [name, type] = package.split('/');
      console.log(
        `Processing versioning [${semvar}:${targetPkg}:${targetType}] for package [${name}] of type [${type}]`,
      );

      if (!fs.existsSync(name)) {
        console.log('could not find dir: ', name);
        continue;
      }

      // cd to package
      cwdArgs.push(name);

      if (shouldUpdate(type, targetType)) {
        cwdArgs.push(type);

        if (isCratesPackage(type)) {
          await updateCratesPackage(io, cwdArgs, name, semvar);
        } else if (isNpmPackage(type)) {
          updateNpmPackage(cwdArgs, name, semvar);
        } else continue;
      } else {
        console.log(`no update required for package = ${name} of type = ${type}`);
        continue;
      }

      // chdir back two levels - back to root, should match original cwd
      cwdArgs.pop();
      cwdArgs.pop();
    }
  }
};
