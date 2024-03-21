use std::result::Result as StdResult;

use thiserror::Error as ThisError;

type Result = StdResult<T, ThisError>;

#[derive(Debug, thisError)]
pub enum Error {}
