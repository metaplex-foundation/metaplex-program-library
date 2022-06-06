// https://github.com/octokit/octokit.graphql.net/blob/master/Octokit.GraphQL/Model/CommentAuthorAssociation.cs
const VALID_AUTHOR_ASSOCIATIONS = ['owner', 'member', 'contributor'];
const VALID_REVIEW_STATES = ['approved', 'commented'];
const VERSIONING_REGEX = /^(patch|minor|major)$/g;

/**
 * Search reviews for first instance with a semvar version command.
 * Reviews are expected to be in reverse chronological order.
 *
 * @param {reviews} list A list of review objects returned from the `listReviews` API
 * @return {string} The semvar version for the review
 */
const findFirstReviewWithVersion = (reviews) => {
  let version = 'none';
  for (const review of reviews) {
    if (!isValidReview(review)) continue;
    const reviewVersion = getVersionFromReview(review);
    if (reviewVersion) {
      // update version and break, we found the most recent version comment
      version = reviewVersion;
      break;
    }
  }

  return version;
};

/**
 * Filters the review body for version update based on `VERSIONING_REGEX`. Returns the last found instance,
 * otherwise null.
 *
 * @param {review} obj A review obj returned from the `listReviews` API
 * @return {string} The semvar version for the review
 */
const getVersionFromReview = (review) => {
  const body = review.body
    .toLowerCase()
    .split('\n')
    .filter((t) => t.length > 0);

  const versionCommentsInBody = body.filter((c) => VERSIONING_REGEX.test(c.trim()));

  // return empty list or last matching version comment
  return versionCommentsInBody.length === 0
    ? null
    : versionCommentsInBody[versionCommentsInBody.length - 1];
};

/**
 * Returns true if a review matches heuristics for a valid review. Otherwise, false.
 *
 * @param {review} obj A review obj returned from the `listReviews` API
 * @return {boolean} Indication of whether or not this review is valid
 */
const isValidReview = (review) => {
  const author_association = review.author_association.toLowerCase();
  const state = review.state.toLowerCase();

  if (!VALID_AUTHOR_ASSOCIATIONS.includes(author_association)) return false;
  if (!VALID_REVIEW_STATES.includes(state)) return false;
  return true;
};

/**
 * Returns all reviews for a given PR in reverse chronological order.
 * The function will iterate through pages to fetch all reviews,
 * with a default of 100 reviews per page.
 *
 * @param {github} obj An @actions/github object
 * @param {owner} string The owners of the repository
 * @param {repo} string The name of the repository
 * @param {pull_number} n The numeric pull requests ID.
 * @param {per_page} n The page size for a given GET request.
 * @return {reviews} The list of reviews associated with this PR
 */
fetchPullRequestReviewsDesc = async (github, owner, repo, pull_number, per_page = 100) => {
  console.log(`Getting PR #${pull_number} from ${owner}/${repo}`);
  let page = 0;
  let reviews = []; // string[]

  while (true) {
    const { data } = await github.pulls.listReviews({
      owner,
      repo,
      pull_number,
      direction: 'asc',
      per_page,
      page,
    });

    if (data.length === 0) break;
    reviews = [
      ...reviews,
      ...data.map((r) => {
        return {
          author_association: r.author_association,
          state: r.state,
          body: r.body,
        };
      }),
    ];

    console.log(`Fetched reviews page: ${page}`);
    page += 1;
  }

  console.log(`Fetched ${reviews.length} reviews for PR ${pull_number}`);
  return reviews.length === 0 ? [] : reviews.reverse();
};

/**
 * Returns all reviews for a given PR. The function will iterate through pages to fetch all reviews, with a default of 100 reviews per page.
 *
 * @param {github} obj An @actions/github object
 * @param {core} obj An @actions/core object
 * @param {pull_number} n The numeric pull request ID
 * @return void
 */
module.exports = async ({ github, context, core }, pull_number) => {
  const reviews = await fetchPullRequestReviewsDesc(
    github,
    context.repo.owner,
    context.repo.repo,
    pull_number,
  );
  const version = findFirstReviewWithVersion(reviews);
  // force js to recognize this as a quoted string for subsequent consumers
  core.exportVariable('REVIEW_VERSION', version);
};
