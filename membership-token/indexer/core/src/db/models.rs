use super::schema::{signatures, transactions};

#[derive(Insertable)]
#[table_name = "signatures"]
pub struct NewSignature<'a> {
    pub signature: &'a str,
    pub slot: i32,
    pub err: &'a str,
    pub memo: &'a str,
    pub block_time: i32,
    pub confirmation_status: &'a str,
    pub loading_status: i32,
}

#[derive(Queryable)]
pub struct Signature {
    pub id: i32,
    pub signature: String,
    pub slot: i32,
    pub err: String,
    pub memo: String,
    pub block_time: i32,
    pub confirmation_status: String,
    pub loading_status: i32,
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction<'a> {
    pub signature: &'a str,
    pub slot: i32,
    pub transaction: &'a str,
    pub block_time: i32,
}
