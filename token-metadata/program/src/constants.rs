/// prefix used for PDAs to avoid certain collision attacks:
/// https://en.wikipedia.org/wiki/Collision_attack#Chosen-prefix_collision_attack

pub const PREFIX: &str = "metadata";

pub const EDITION: &str = "edition";

pub const RESERVATION: &str = "reservation";

pub const USER: &str = "user";

pub const BURN: &str = "burn";

pub const COLLECTION_AUTHORITY: &str = "collection_authority";

pub const ESCROW_POSTFIX: &str = "escrow";

// Size constants.

pub const MAX_NAME_LENGTH: usize = 32;

pub const MAX_SYMBOL_LENGTH: usize = 10;

pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_METADATA_LEN: usize = 1 // key 
+ 32             // update auth pubkey
+ 32             // mint pubkey
+ MAX_DATA_SIZE
+ 1              // primary sale
+ 1              // mutable
+ 9              // nonce (pretty sure this only needs to be 2)
+ 2              // token standard
+ 34             // collection
+ 18             // uses
+ 118; // Padding

pub const MAX_DATA_SIZE: usize = 4
    + MAX_NAME_LENGTH
    + 4
    + MAX_SYMBOL_LENGTH
    + 4
    + MAX_URI_LENGTH
    + 2
    + 1
    + 4
    + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN;

pub const MAX_EDITION_LEN: usize = 1 + 32 + 8 + 200;

// Large buffer because the older master editions have two pubkeys in them,
// need to keep two versions same size because the conversion process actually
// changes the same account by rewriting it.
pub const MAX_MASTER_EDITION_LEN: usize = 1 + 9 + 8 + 264;

pub const MAX_CREATOR_LIMIT: usize = 5;

pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

pub const MAX_RESERVATIONS: usize = 200;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_V1_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 34 + 100;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 48 + 8 + 8 + 84;

pub const MAX_EDITION_MARKER_SIZE: usize = 32;

pub const EDITION_MARKER_BIT_SIZE: u64 = 248;

pub const USE_AUTHORITY_RECORD_SIZE: usize = 18; //8 byte padding

pub const COLLECTION_AUTHORITY_RECORD_SIZE: usize = 35;
