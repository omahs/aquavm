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

use super::select_by_lambda_from_scalar;
use super::ExecutionResult;
use super::JValuable;
use super::LambdaAST;
use super::ValueAggregate;
use crate::execution_step::value_types::populate_tetraplet_with_lambda;
use crate::execution_step::ExecutionCtx;
use crate::execution_step::RcSecurityTetraplets;
use crate::JValue;
use crate::SecurityTetraplet;

use air_interpreter_data::Provenance;

use std::borrow::Cow;
use std::ops::Deref;

impl JValuable for ValueAggregate {
    fn apply_lambda(&self, lambda: &LambdaAST<'_>, exec_ctx: &ExecutionCtx<'_>) -> ExecutionResult<Cow<'_, JValue>> {
        let selected_value = select_by_lambda_from_scalar(self.get_result(), lambda, exec_ctx)?;
        Ok(selected_value)
    }

    fn apply_lambda_with_tetraplets(
        &self,
        lambda: &LambdaAST<'_>,
        exec_ctx: &ExecutionCtx<'_>,
        _root_provenane: &Provenance,
    ) -> ExecutionResult<(Cow<'_, JValue>, SecurityTetraplet, Provenance)> {
        let selected_value = select_by_lambda_from_scalar(self.get_result(), lambda, exec_ctx)?;
        let tetraplet = populate_tetraplet_with_lambda(self.get_tetraplet().as_ref().clone(), lambda);

        Ok((selected_value, tetraplet, self.get_provenance()))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        Cow::Borrowed(self.get_result())
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        self.get_result().deref().clone()
    }

    fn as_tetraplets(&self) -> RcSecurityTetraplets {
        vec![self.get_tetraplet()]
    }
}
