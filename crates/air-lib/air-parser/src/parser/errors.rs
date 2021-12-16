/*
 * Copyright 2020 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::parser::lexer::LexerError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    #[error(transparent)]
    LexerError(#[from] LexerError),

    #[error("lambda can't be applied to streams in this position")]
    LambdaAppliedToStream(usize, usize),

    #[error("variable '{2}' wasn't defined")]
    UndefinedVariable(usize, usize, String),

    #[error("iterable '{2}' wasn't defined")]
    UndefinedIterable(usize, usize, String),

    #[error("last error with non-empty path is ambiguous, please use just %last_error%")]
    AmbiguousFailLastError(usize, usize),

    /// Semantic errors in a call instructions.
    #[error("call should have service id specified by peer part or function part")]
    InvalidCallTriplet(usize, usize),

    #[error("new can't be applied to a '{2}' because it's an iterator")]
    IteratorRestrictionNotAllowed(usize, usize, String),
}

impl From<std::convert::Infallible> for ParserError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
