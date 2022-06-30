const fs = require('fs');
const path = require('path');
const toml = require('@iarna/toml');
const { execSync } = require('child_process');

// todo: move somewhere, like a separate config/constants file.
const MPL_PROGRAM_CONFIG = {
  'auction-house': {
    has_idl: true,
    uses_anchor: true,
  },
  auction: {
    has_idl: false,
    uses_anchor: false,
  },
  auctioneer: {
    has_idl: true,
    uses_anchor: true,
  },
  core: {
    has_idl: false,
    uses_anchor: false,
  },
  'candy-machine': {
    has_idl: true,
    uses_anchor: true,
  },
  'fixed-price-sale': {
    has_idl: true,
    uses_anchor: true,
  },
  gumdrop: {
    has_idl: false,
    uses_anchor: false,
  },
  metaplex: {
    has_idl: false,
    uses_anchor: false,
  },
  'nft-packs': {
    has_idl: false,
    uses_anchor: false,
  },
  'token-entangler': {
    has_idl: true,
    uses_anchor: true,
  },
  // uses shank
  'token-metadata': {
    has_idl: true,
    uses_anchor: false,
  },
  // uses shank
  'token-vault': {
    has_idl: true,
    uses_anchor: false,
  },
};

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

const packageUsesAnchor = (pkg) => {
  const result = MPL_PROGRAM_CONFIG[pkg]['uses_anchor'];
  console.log(`${pkg} uses anchor: ${result}`);
  return result;
};

const packageHasIdl = (pkg) => {
  const result = MPL_PROGRAM_CONFIG[pkg]['has_idl'];
  console.log(`${pkg} has idl: ${result}`);
  return result;
};

const isPackageType = (actual, target) => actual === target;
// additional equality checks can match other subdirs, e.g. `rust|test|cli|<etc>`
const isCratesPackage = (actual) => isPackageType(actual, 'program');
const isNpmPackage = (actual) => isPackageType(actual, 'js');

const parseVersioningCommand = (cmd) => cmd.split(':');
const shouldUpdate = (actual, target) => target === '*' || target === actual;

const getCrateVersion = (cargoPath) => {
  let tomlObj = toml.parse(fs.readFileSync(cargoPath, 'utf-8'));
  if (!tomlObj.package) throw new Error('No package tag defined in Cargo.toml');

  return tomlObj.package.version;
};

const updateIdlWithVersion = (idlPath, cargoPath) => {
  console.log('pulling version from ', cargoPath);
  const crateVersion = getCrateVersion(cargoPath);

  if (!fs.existsSync(idlPath)) {
    throw new Error(`cannot find IDL: ${idlPath}`);
  }

  var idl = JSON.parse(fs.readFileSync(idlPath, 'utf8'));
  console.log(`updating IDL at ${idlPath} from ${idl['version']} to ${crateVersion}`);
  idl['version'] = crateVersion;

  fs.writeFileSync(idlPath, JSON.stringify(idl, null, 2));
};

const updateCratesPackage = (cwdArgs, pkg, semvar) => {
  console.log('updating rust package');
  const currentDir = cwdArgs.join('/');

  // adds git info automatically
  wrappedExec(
    `cargo release --no-publish --no-push --no-confirm --verbose --execute --no-verify --no-tag --config ../../release.toml ${semvar}`,
    currentDir,
  );

  if (packageHasIdl(pkg)) {
    // replace all instances of - with _
    let idlName = `${pkg.replace(/\-/g, '_')}.json`;
    if (!packageUsesAnchor(pkg)) {
      console.log('package uses shank-cli for IDL');
      idlName = `mpl_${idlName}`;
    }
    console.log('final IDL name: ', idlName);

    // create ../js/idl dir if it does not exist; back one dir + js dir + idl dir
    // note: cwdArgs == currentDir.split("/")
    const destIdlDir = [...cwdArgs.slice(0, cwdArgs.length - 1), 'js', 'idl'].join('/');

    updateIdlWithVersion(`${destIdlDir}/${idlName}`, `${currentDir}/Cargo.toml`);
    // append IDL change to rust version bump commit
    wrappedExec(`git add ${destIdlDir} && git commit --amend -C HEAD && git push`);
  }
};

const updateNpmPackage = (cwdArgs, _pkg, semvar) => {
  wrappedExec('cat ../../package.json', cwdArgs.join('/'));

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
 * @param {packages} arr List of packages to process in the form <pkg-name>/<sub-dir>
 * @param {versioning} arr List of version commands in the form semvar:pkg:type where type = `program|js`
 * @return void
 */
module.exports = async (packages, versioning) => {
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

    for (let package of JSON.parse(packages)) {
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
          updateCratesPackage(cwdArgs, name, semvar);
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
