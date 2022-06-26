// https://github.com/octokit/octokit.graphql.net/blob/master/Octokit.GraphQL/Model/CommentAuthorAssociation.cs
const SEMVAR_COMMAND = /^(patch|minor|major)$/;
const VERSIONING_COMMAND = /^version/g;

const parse = (body) => {
  const trimmedBody = body
    .toLowerCase()
    .split("\n")
    .filter((t) => t.length > 0);

  const validVersionCmds = trimmedBody.filter((c) =>
    VERSIONING_COMMAND.test(c.trim())
  );

  if (validVersionCmds.length === 0) {
    console.log("no valid version commands");
    return []; // emtpy list
  }

  console.log(
    `found ${validVersionCmds.length} version commands. only the first will be processed.`
  );
  // does \s+ or \w+ work in js
  const cmd = validVersionCmds[0].split(" ").slice(1);

  if (!SEMVAR_COMMAND.test(cmd)) {
    throw new Error("Invalid command: ", cmd);
  }

  // formatted for => 0: semvar, 1: package, 2: language
  return [[cmd, "*", "*"].join(":")];
};

// in the future, we can
module.exports = async ({ github, context, core }, body) => {
  const versioning = parse(body);
  console.log(versioning);

  // explicit formatting for CI
  core.exportVariable(
    "VERSIONING",
    versioning.map((v) => `\"${v}\"`)
  );
  core.exportVariable("HAS_VERSIONING", versioning.length > 0);
};
