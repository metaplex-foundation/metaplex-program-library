pub mod solana;
pub mod utils;

#[macro_export]
macro_rules! assert_transport_error {
    ($error:expr, $matcher:pat) => {
        match $error {
            $matcher => {
                assert!(true)
            }
            _ => assert!(false),
        }
    };
}

#[macro_export]
macro_rules! assert_error {
    ($error:expr, $matcher:expr) => {
        match $error {
            TransportError::TransactionError(TransactionError::InstructionError(
                0,
                InstructionError::Custom(x),
            )) => assert_eq!(x, $matcher),
            _ => assert!(false),
        };
    };
}
