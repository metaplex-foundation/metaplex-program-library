const fetchAllChangedFiles = async (
  github,
  owner,
  repo,
  pull_number,
  include, // [] | undefined
  exclude, // [] | undefined
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

  let result = Array.from(files);
  if (include) {
    // it's possible exclude is a stringified arr
    if (typeof include === 'string') {
      include = JSON.parse(include);
    }
    console.log('before include: ', result);
    result = result.filter((f) => {
      return include.reduce((prev, path) => {
        return prev || f.includes(path);
      }, false);
    });
    console.log('after include: ', result);
  }

  if (exclude) {
    // it's possible exclude is a stringified arr
    if (typeof exclude === 'string') {
      exclude = JSON.parse(exclude);
    }
    console.log('before exclude: ', result);
    result = result.filter((f) => {
      return exclude.reduce((prev, path) => {
        return prev && !f.includes(path);
      }, true);
    });
    console.log('after exclude: ', result);
  }

  return result;
};

module.exports = async ({ github, context, core }, pull_number, include, exclude) => {
  const changedFiles = await fetchAllChangedFiles(
    github,
    context.repo.owner,
    context.repo.repo,
    pull_number,
    include,
    exclude,
  );

  core.exportVariable(
    'CHANGED_FILES',
    // explicitly add quotation marks for later parsing
    JSON.stringify(Array.from(changedFiles).map((el) => `\"${el}\"`)),
  );
};
