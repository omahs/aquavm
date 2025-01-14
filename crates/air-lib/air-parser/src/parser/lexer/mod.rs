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

mod air_lexer;
mod call_variable_parser;
mod errors;
mod token;
mod utils;

#[cfg(test)]
mod tests;
pub mod text_pos;

pub use air_lexer::AIRLexer;
pub(crate) use air_lexer::ERROR;
pub(crate) use air_lexer::LAST_ERROR;
pub use errors::LexerError;
pub use text_pos::AirPos;
pub use token::Token;

pub(super) type LexerResult<T> = std::result::Result<T, LexerError>;

use utils::is_air_alphanumeric;
use utils::is_json_path_allowed_char;
