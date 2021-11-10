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

use marine_rs_sdk::marine;

use fluence_it_types::IValue;
use serde::Deserialize;
use serde::Serialize;

pub const INTERPRETER_SUCCESS: i32 = 0;

/// Describes a result returned at the end of the interpreter execution_step.
#[marine]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpreterOutcome {
    /// A return code, where INTERPRETER_SUCCESS means success.
    pub ret_code: i32,

    /// Contains error message if ret_code != INTERPRETER_SUCCESS.
    pub error_message: String,

    /// Contains script data that should be preserved in an executor of this interpreter
    /// regardless of ret_code value.
    pub data: Vec<u8>,

    /// Public keys of peers that should receive data.
    pub next_peer_pks: Vec<String>,

    /// Collected parameters of all met call instructions that could be executed on a current peer.
    pub call_requests: Vec<u8>,
}

impl InterpreterOutcome {
    pub fn from_ivalue(ivalue: IValue) -> Result<Self, String> {
        const OUTCOME_FIELDS_COUNT: usize = 5;

        let mut record_values = try_as_record(ivalue)?.into_vec();
        if record_values.len() != OUTCOME_FIELDS_COUNT {
            return Err(format!(
                "expected InterpreterOutcome struct with {} fields, got {:?}",
                OUTCOME_FIELDS_COUNT, record_values
            ));
        }

        let call_requests = try_as_byte_vec(record_values.pop().unwrap(), "call_requests")?;
        let next_peer_pks = try_as_string_vec(record_values.pop().unwrap(), "next_peer_pks")?;
        let data = try_as_byte_vec(record_values.pop().unwrap(), "data")?;
        let error_message = try_as_string(record_values.pop().unwrap(), "error_message")?;
        let ret_code = try_as_i32(record_values.pop().unwrap(), "ret_code")?;

        let outcome = Self {
            ret_code,
            error_message,
            data,
            next_peer_pks,
            call_requests,
        };

        Ok(outcome)
    }
}

use fluence_it_types::ne_vec::NEVec;

fn try_as_record(ivalue: IValue) -> Result<NEVec<IValue>, String> {
    match ivalue {
        IValue::Record(record_values) => Ok(record_values),
        v => {
            return Err(format!(
                "expected record for InterpreterOutcome, got {:?}",
                v
            ))
        }
    }
}

fn try_as_i32(ivalue: IValue, field_name: &str) -> Result<i32, String> {
    match ivalue {
        IValue::S32(value) => Ok(value),
        v => return Err(format!("expected an i32 for {}, got {:?}", field_name, v)),
    }
}

fn try_as_string(ivalue: IValue, field_name: &str) -> Result<String, String> {
    match ivalue {
        IValue::String(value) => Ok(value),
        v => return Err(format!("expected a string for {}, got {:?}", field_name, v)),
    }
}

fn try_as_byte_vec(ivalue: IValue, field_name: &str) -> Result<Vec<u8>, String> {
    let byte_vec = match ivalue {
        IValue::Array(array) => {
            let array: Result<Vec<_>, _> = array
                .into_iter()
                .map(|v| match v {
                    IValue::U8(byte) => Ok(byte),
                    v => Err(format!("expected a byte, got {:?}", v)),
                })
                .collect();
            array?
        }
        IValue::ByteArray(array) => array,
        v => {
            return Err(format!(
                "expected a Vec<u8> for {}, got {:?}",
                field_name, v
            ))
        }
    };

    Ok(byte_vec)
}

fn try_as_string_vec(ivalue: IValue, field_name: &str) -> Result<Vec<String>, String> {
    match ivalue {
        IValue::Array(ar_values) => {
            let array = ar_values
                .into_iter()
                .map(|v| match v {
                    IValue::String(str) => Ok(str),
                    v => Err(format!("expected string for next_peer_pks, got {:?}", v)),
                })
                .collect::<Result<Vec<String>, _>>()?;

            Ok(array)
        }
        v => Err(format!("expected an array for {}, got {:?}", field_name, v)),
    }
}