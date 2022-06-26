const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

// store somewhere else?
const MPL_PROGRAM_CONFIG = {
  newworld: {
    has_idl: "true",
    uses_anchor: "true",
  },
  "auction-house": {
    has_idl: "true",
    uses_anchor: "true",
  },
  auction: {
    has_idl: "false",
    uses_anchor: "false",
  },
  auctioneer: {
    has_idl: "true",
    uses_anchor: "true",
  },
  core: {
    has_idl: "false",
    uses_anchor: "false",
  },
  "candy-machine": {
    has_idl: "true",
    uses_anchor: "true",
  },
  "fixed-price-sale": {
    has_idl: "true",
    uses_anchor: "true",
  },
  gumdrop: {
    has_idl: "false",
    uses_anchor: "false",
  },
  metaplex: {
    has_idl: "false",
    uses_anchor: "false",
  },
  "nft-packs": {
    has_idl: "false",
    uses_anchor: "false",
  },
  "token-entangler": {
    has_idl: "true",
    uses_anchor: "true",
  },
  // uses shank
  "token-metadata": {
    has_idl: "true",
    uses_anchor: "false",
  },
  // uses shank
  "token-vault": {
    has_idl: "true",
    uses_anchor: "false",
  },
};

const wrappedExec = (cmd, cwd) => {
  let args = {
    stdio: "inherit",
  };

  if (cwd) {
    args["cwd"] = cwd;
  } else {
    args["cwd"] = path.join(__dirname);
  }

  execSync(cmd, args);
};

const isPackageType = (actual, target) => actual === target;
const isCratesPackage = (actual) => isPackageType(actual, "program");
const isNpmPackage = (actual) => isPackageType(actual, "js");

const packageUsesAnchor = (pkg) => MPL_PROGRAM_CONFIG[pkg]["uses_anchor"];
const packageHasIdl = (pkg) => MPL_PROGRAM_CONFIG[pkg]["has_idl"];

const parseVersioningCommand = (cmd) => cmd.split(":");
const shouldUpdate = (actual, target) => target === "*" || target === actual;

const updateCratesPackage = async (io, cwdArgs, pkg, semvar) => {
  console.log("updating rust package");
  const currentDir = cwdArgs.join("/");

  // adds git info automatically
  wrappedExec(
    `cargo release --no-publish --no-push --no-confirm --verbose --execute --no-tag ${semvar}`,
    currentDir
  );
  wrappedExec(`git log`);
  // // wrappedExec(`shank --help`);

  console.log("=====================");
  wrappedExec("pwd", currentDir);
  console.log("=====================");

  const sourceIdlDir = [
    ...cwdArgs.slice(0, cwdArgs.length - 2),
    "target",
    "idl",
  ].join("/");

  // generate IDL
  if (packageHasIdl(pkg)) {
    let idlName = `${pkg.replace("-", "_")}.json`;
    if (packageUsesAnchor(pkg)) {
      console.log("generate IDL via anchor");
      // build via anchor to generate IDL
      wrappedExec(`anchor build --skip-lint --idl ${sourceIdlDir}`, currentDir);
    } else {
      console.log("generate IDL via shank");
      // generate IDL via shank
      // todo: test shank command in mpl
      wrappedExec(
        `shank idl --out-dir ${sourceIdlDir}  --crate-root .`,
        currentDir
      );
      // prepend `mpl_` to IDL name
      idlName = `mpl_${idlName}`;
    }

    // create ../js/idl dir if it does not exist; back one dir + js dir + idl dir
    // note: cwdArgs == currentDir.split("/")
    const destIdlDir = [
      ...cwdArgs.slice(0, cwdArgs.length - 1),
      "js",
      "idl",
    ].join("/");

    if (!fs.existsSync(destIdlDir)) {
      console.log(`creating ${destIdlDir}`);
      await io.mkdirP(destIdlDir);
    }

    console.log("=====================");
    wrappedExec(`ls ${destIdlDir}`, currentDir);
    console.log("=====================");

    console.log("idlName: ", idlName);
    // cp IDL to js dir
    wrappedExec(`cp ${sourceIdlDir}/${idlName} ${destIdlDir}`, currentDir);

    console.log("=====================");
    wrappedExec(`ls ${destIdlDir}`, currentDir);
    console.log("=====================");

    console.log("=====================");
    // append IDL change to rust version bump commit
    wrappedExec(`git add -A && git commit --amend -C HEAD`);
    console.log("=====================");
    wrappedExec(`git log`);
    console.log("=====================");
    wrappedExec(`git diff HEAD~1 HEAD`);
    console.log("=====================");
  }
};

const updateNpmPackage = (cwdArgs, _pkg, semvar) => {
  console.log("updating js package");

  // adds git info automatically
  wrappedExec("yarn install", cwdArgs.join("/"));
  wrappedExec(`npm version ${semvar}`, cwdArgs.join("/"));
  console.log("log after upate: ", wrappedExec("git log"));
};

// todo: add comment for expected format
module.exports = async (
  { github, context, core, glob, io },
  packages,
  versioning
) => {
  // versioning = JSON.parse(versioning);

  const base = process.env.GITHUB_ACTION_PATH; // path.join(__dirname);
  // ./.github/actions/<name>
  const splitBase = base.split("/");
  const cwdArgs = splitBase.slice(0, splitBase.length - 4);
  console.log("cwdArgs: ", cwdArgs);
  console.log(`===========================`);

  wrappedExec("ls", cwdArgs.join("/"));

  console.log("versioning: ", versioning);

  if (versioning.length === 0) {
    console.log("No versioning updates to make. Exiting early.");
    return;
  }

  wrappedExec("git config user.name github-actions[bot]", cwdArgs.join("/"));
  wrappedExec(
    "git config user.email github-actions[bot]@users.noreply.github.com",
    cwdArgs.join("/")
  );

  // packages   => [auction-house/program, candy-machine/js]
  // versioning => ["patch"]

  // for each versioning, check if applies to package?
  for (const version of versioning) {
    console.log("version: ", version);
    const [semvar, targetPkg, targetType] = parseVersioningCommand(version);
    if (semvar === "none") {
      console.log(
        "No versioning updates to make when semvar === none. Continuing."
      );
      continue;
    }

    console.log("semvar: ", semvar);
    console.log("targetPkg: ", targetPkg);
    console.log("targetType: ", targetType);

    console.log("packages: ", packages);

    packages = JSON.parse(packages);
    for (let package of packages) {
      // make sure package doesn't have extra quotes or spacing
      package = package.replace(/\s+|\"|\'/g, "");
      console.log("package: ", package);

      if (!shouldUpdate(package, targetPkg)) {
        console.log(
          `No updates for package ${package} based on version command ${version}`
        );
        continue;
      }

      const [name, type] = package.split("/");
      console.log("name: ", name);
      console.log("type: ", type);

      if (!fs.existsSync(name)) {
        console.log("could not find dir: ", name);
        continue;
      }

      // cd to package
      console.log(`cd to package: ${name}`);
      cwdArgs.push(name);

      if (shouldUpdate(type, targetType)) {
        console.log(`add type to cwd: ${type}`);
        cwdArgs.push(type);

        if (isCratesPackage(type))
          await updateCratesPackage(io, cwdArgs, name, semvar);
        else if (isNpmPackage(type)) updateNpmPackage(cwdArgs, name, semvar);
        else continue;
      } else {
        console.log(
          `no update required for package = ${name} of type = ${type}`
        );
        continue;
      }

      // chdir back two levels - back to root, should match original cwd
      console.log("remove 2 args to go back 2 dirs");
      cwdArgs.pop();
      cwdArgs.pop();
    }
  }
};
