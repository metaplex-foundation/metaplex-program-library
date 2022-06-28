// add to action input params at some point
const PATHS_TO_IGNORE = [
  '.github',
  'Cargo.lock',
  'Cargo.toml',
  'js/idl',
  'packge.json',
  'yarn.lock',
];

const fetchAllChangedFiles = async (
  github,
  owner,
  repo,
  pull_number,
  excludePaths = [],
  per_page = 100,
) => {
  let page = 0;
  let files = new Set();

  while (true) {
    const { data } = await github.pulls.listFiles({
      owner,
      repo,
      pull_number,
      direction: 'asc',
      per_page,
      page,
    });

    if (data.length === 0) break;
    data.map((f) => f.filename).forEach((f) => files.add(f));
    console.log(`fetched page ${page}`);
    page += 1;
  }

  return Array.from(files).filter((f) => {
    return excludePaths.reduce((prev, path) => {
      return prev && !f.includes(path);
    }, true);
  });
};

module.exports = async ({ github, context, core }, pull_number) => {
  const changedFiles = await fetchAllChangedFiles(
    github,
    context.repo.owner,
    context.repo.repo,
    pull_number,
    PATHS_TO_IGNORE,
  );

  core.exportVariable(
    'CHANGED_FILES',
    // explicitly add quotation marks for later parsing
    JSON.stringify(Array.from(changedFiles).map((el) => `\"${el}\"`)),
  );
};
