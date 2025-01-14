/*
 * Copyright 2021 Fluence Labs Limited
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

use super::ErrorObjectError;
use crate::execution_step::RcSecurityTetraplet;
use crate::JValue;

use air_interpreter_data::Provenance;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use std::rc::Rc;

pub const ERROR_CODE_FIELD_NAME: &str = "error_code";
pub const MESSAGE_FIELD_NAME: &str = "message";
pub const INSTRUCTION_FIELD_NAME: &str = "instruction";
pub const PEER_ID_FIELD_NAME: &str = "peer_id";
pub const NO_ERROR_MESSAGE: &str = "";
pub const NO_ERROR_ERROR_CODE: i64 = 0;

/// This struct tracks the last arisen error.
/// :error: and %last_error% are special scalar values that support lenses.
/// There are some differences b/w mentioned errors and an ordinary scalar:
///  - they are set to not-an-error value by default
///  - these are global scalars, meaning that fold and new scopes do not apply for it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionError {
    /// Error object that represents the last occurred error.
    pub error: Rc<JValue>,

    /// Tetraplet that identify host where the error occurred.
    pub tetraplet: Option<RcSecurityTetraplet>,

    /// Value provenance
    pub provenance: Provenance,
}

pub(crate) fn error_from_raw_fields_w_peerid(
    error_code: i64,
    error_message: &str,
    instruction: &str,
    peer_id: &str,
) -> JValue {
    serde_json::json!({
        ERROR_CODE_FIELD_NAME: error_code,
        MESSAGE_FIELD_NAME: error_message,
        INSTRUCTION_FIELD_NAME: instruction,
        PEER_ID_FIELD_NAME: peer_id,
    })
}

pub(crate) fn error_from_raw_fields(error_code: i64, error_message: &str, instruction: &str) -> JValue {
    serde_json::json!({
        ERROR_CODE_FIELD_NAME: error_code,
        MESSAGE_FIELD_NAME: error_message,
        INSTRUCTION_FIELD_NAME: instruction,
    })
}

/// Checks that a scalar is a value of an object types that contains at least two fields:
///  - error_code != 0
///  - message
pub(crate) fn check_error_object(scalar: &JValue) -> Result<(), ErrorObjectError> {
    let fields = match scalar {
        JValue::Object(fields) => fields,
        _ => return Err(ErrorObjectError::ScalarMustBeObject(scalar.clone())),
    };

    let check_field = |field_name| {
        fields
            .get(field_name)
            .ok_or_else(|| ErrorObjectError::ScalarMustContainField {
                scalar: scalar.clone(),
                field_name,
            })
    };

    let error_code = check_field(ERROR_CODE_FIELD_NAME)?;
    ensure_error_code_correct(scalar, error_code, ERROR_CODE_FIELD_NAME)?;

    let message = check_field(MESSAGE_FIELD_NAME)?;
    ensure_jvalue_is_string(scalar, message, MESSAGE_FIELD_NAME)?;

    Ok(())
}

fn ensure_error_code_correct(
    scalar: &JValue,
    value: &JValue,
    field_name: &'static str,
) -> Result<(), ErrorObjectError> {
    match value {
        JValue::Number(number) if number.is_i64() | number.is_u64() => {
            ensure_error_code_is_error(number.as_i64().unwrap())
        }
        _ => Err(ErrorObjectError::ScalarFieldIsWrongType {
            scalar: scalar.clone(),
            field_name,
            expected_type: "integer",
        }),
    }
}

fn ensure_error_code_is_error(number: i64) -> Result<(), ErrorObjectError> {
    if number == NO_ERROR_ERROR_CODE {
        Err(ErrorObjectError::ErrorCodeMustBeNonZero)
    } else {
        Ok(())
    }
}

fn ensure_jvalue_is_string(scalar: &JValue, value: &JValue, field_name: &'static str) -> Result<(), ErrorObjectError> {
    match value {
        JValue::String(_) => Ok(()),
        _ => Err(ErrorObjectError::ScalarFieldIsWrongType {
            scalar: scalar.clone(),
            field_name,
            expected_type: "string",
        }),
    }
}

pub fn no_error_object() -> JValue {
    json!({
        ERROR_CODE_FIELD_NAME: NO_ERROR_ERROR_CODE,
        MESSAGE_FIELD_NAME: NO_ERROR_MESSAGE,
    })
}

pub fn no_error() -> InstructionError {
    InstructionError {
        error: Rc::new(no_error_object()),
        tetraplet: None,
        provenance: Provenance::literal(),
    }
}
