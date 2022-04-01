use mpl_auction_house::AuthorityScope;

pub fn default_scopes() -> Vec<AuthorityScope> {
    vec![
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
    ]
}
