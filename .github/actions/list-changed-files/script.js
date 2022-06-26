const getPackageAndExtension = (path) => {
  const invalidPathDefault = ["", ""];

  const pathParts = path.trim().toLowerCase().split("/");
  if (pathParts.length === 0) return invalidPathDefault;
  const extensionParts = pathParts[pathParts.length - 1].split(".");
  // we expect at least 2 items in the result: ['name', 'ending']
  if (extensionParts.length < 2) return invalidPathDefault;

  // [package, extension]
  return [pathParts[0], extensionParts[1]];
};

fetchAllChangedFiles = async (
  github,
  owner,
  repo,
  pull_number,
  per_page = 100 // max = 100?
) => {
  let page = 0;
  let files = [];

  while (true) {
    const { data } = await github.pulls.listFiles({
      owner,
      repo,
      pull_number,
      direction: "asc",
      per_page,
      page,
    });
    console.log(`fetched page ${page}`);

    // break early if we received no results
    if (data.length === 0) break;
    files = [...files, ...data.map((f) => f.filename)];
    // break early if we received fewer results than the max
    if (data.length < per_page) break;

    page += 1;
  }

  console.log(`Fetched ${files.length} files for PR ${pull_number}`);

  return files;
};

module.exports = async ({ github, context, core }, pull_number) => {
  const changedFiles = await fetchAllChangedFiles(
    github,
    context.repo.owner,
    context.repo.repo,
    pull_number
  );

  console.log("num changedFiles: ", changedFiles.length);
  console.log("changedFiles: ", changedFiles);

  core.exportVariable(
    "CHANGED_FILES",
    // explicitly add quotation marks for later parsing
    JSON.stringify(Array.from(changedFiles).map((el) => `\"${el}\"`))
  );
};
