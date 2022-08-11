use mpl_auction_house::{constants::MAX_NUM_SCOPES, AuthorityScope};

pub fn default_scopes() -> Vec<AuthorityScope> {
    vec![
        AuthorityScope::Deposit,
        AuthorityScope::Buy,
        AuthorityScope::PublicBuy,
        AuthorityScope::ExecuteSale,
        AuthorityScope::Sell,
        AuthorityScope::Cancel,
        AuthorityScope::Withdraw,
    ]
}

pub fn assert_scopes_eq(scopes: Vec<AuthorityScope>, scopes_array: [bool; MAX_NUM_SCOPES]) {
    for scope in scopes {
        if !scopes_array[scope as usize] {
            panic!();
        }
    }
}
