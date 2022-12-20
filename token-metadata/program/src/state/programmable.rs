use num_derive::ToPrimitive;

#[derive(ToPrimitive)]
pub enum Operation {
    Delegate,
    Transfer,
    DelegatedTransfer,
    MigrateClass,
}
