/*
 * Copyright 2023 Fluence Labs Limited
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

use super::ExecutionResult;
use super::JValuable;
use crate::execution_step::lambda_applier::select_by_lambda_from_canon_map;
use crate::execution_step::value_types::CanonStreamMap;
use crate::execution_step::ExecutionCtx;
use crate::execution_step::RcSecurityTetraplets;
use crate::JValue;
use crate::LambdaAST;
use crate::SecurityTetraplet;

use air_interpreter_data::Provenance;

use std::borrow::Cow;

impl JValuable for &CanonStreamMap<'_> {
    fn apply_lambda(&self, lambda: &LambdaAST<'_>, exec_ctx: &ExecutionCtx<'_>) -> ExecutionResult<Cow<'_, JValue>> {
        select_by_lambda_from_canon_map(self, lambda, exec_ctx)
    }

    fn apply_lambda_with_tetraplets(
        &self,
        lambda: &LambdaAST<'_>,
        exec_ctx: &ExecutionCtx<'_>,
        root_provenance: &Provenance,
    ) -> ExecutionResult<(Cow<'_, JValue>, SecurityTetraplet, Provenance)> {
        let value = select_by_lambda_from_canon_map(self, lambda, exec_ctx)?;

        // TODO improve tetraplet granularity for a canon map with a lens. See VM-331 for the details.
        // Canon map result is a canon stream rendered on this node thus its tetraplet
        // is also calculated and canon map provenance is borrowed.
        let tetraplet = SecurityTetraplet::new(self.tetraplet().peer_pk.to_owned(), "", "", lambda.to_string());

        Ok((value, tetraplet, root_provenance.clone()))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        let jvalue = CanonStreamMap::as_jvalue(self);
        Cow::Owned(jvalue)
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        CanonStreamMap::as_jvalue(&self)
    }

    fn as_tetraplets(&self) -> RcSecurityTetraplets {
        self.iter().map(|r| r.get_tetraplet()).collect()
    }
}
