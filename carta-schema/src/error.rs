use failure_derive::Fail;

#[derive(Fail, Debug, PartialEq)]
#[fail(display = "Error!")]
pub struct CartaError;
