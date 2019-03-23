use failure_derive::Fail;

#[derive(Fail, Debug)]
#[fail(display = "Error!")]
pub struct CartaError;
