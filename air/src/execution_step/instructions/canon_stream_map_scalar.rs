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

use super::canon_utils::handle_seen_canon;
use super::canon_utils::handle_unseen_canon;
use super::canon_utils::CanonEpilogClosure;
use super::canon_utils::CreateCanonStreamClosure;
use super::ExecutionCtx;
use super::ExecutionResult;
use super::TraceHandler;
use crate::execution_step::value_types::CanonStream;
use crate::execution_step::CanonResultAggregate;
use crate::execution_step::LiteralAggregate;
use crate::execution_step::ValueAggregate;
use crate::log_instruction;
use crate::trace_to_exec_err;
use crate::JValue;
use crate::UncatchableError;

use air_interpreter_cid::CID;
use air_interpreter_data::CanonResult;
use air_interpreter_data::CanonResultCidAggregate;
use air_parser::ast;
use air_parser::AirPos;
use air_trace_handler::merger::MergerCanonResult;

use std::borrow::Cow;
use std::rc::Rc;

impl<'i> super::ExecutableInstruction<'i> for ast::CanonStreamMapScalar<'i> {
    #[tracing::instrument(level = "debug", skip(exec_ctx, trace_ctx))]
    fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut TraceHandler) -> ExecutionResult<()> {
        log_instruction!(canon, exec_ctx, trace_ctx);
        let epilog = &epilog_closure(self.scalar.name);
        let canon_result = trace_to_exec_err!(trace_ctx.meet_canon_start(), self)?;

        let create_canon_producer = create_canon_stream_producer(self.stream_map.name, self.stream_map.position);
        match canon_result {
            MergerCanonResult::CanonResult(canon_result) => handle_seen_canon(
                epilog,
                &create_canon_producer,
                canon_result,
                &self.peer_id,
                exec_ctx,
                trace_ctx,
            ),
            MergerCanonResult::Empty => {
                handle_unseen_canon(epilog, &create_canon_producer, &self.peer_id, exec_ctx, trace_ctx)
            }
        }
    }
}

fn epilog_closure<'closure, 'name: 'closure>(scalar_name: &'name str) -> Box<CanonEpilogClosure<'closure>> {
    Box::new(
        move |canon_stream: CanonStream,
              canon_result_cid: Rc<CID<CanonResultCidAggregate>>,
              exec_ctx: &mut ExecutionCtx<'_>,
              trace_ctx: &mut TraceHandler|
              -> ExecutionResult<()> {
            use crate::CanonStreamMapError::NoDataToProduceScalar;

            let tetraplet = canon_stream.tetraplet().clone();
            let peer_pk = tetraplet.peer_pk.as_str().into();

            // Here canon_stream is a transport that brings a single JValue rendered
            // by the producer closure previously.
            let value = canon_stream
                .into_values()
                .first()
                .ok_or(UncatchableError::CanonStreamMapError(NoDataToProduceScalar))?
                .get_result()
                .clone();

            let position = trace_ctx.trace_pos().map_err(UncatchableError::from)?;

            let value = CanonResultAggregate::new(value, peer_pk, &tetraplet.json_path, position);
            let result = ValueAggregate::from_canon_result(value, canon_result_cid.clone());

            exec_ctx.scalars.set_scalar_value(scalar_name, result)?;
            trace_ctx.meet_canon_end(CanonResult::executed(canon_result_cid));
            Ok(())
        },
    )
}

/// The result closure creates canon stream based on the underlying stream or an empty stream
/// if no stream created yet. The latter is crucial for deterministic behaviour, for more info see
/// github.com/fluencelabs/aquavm/issues/346.
fn create_canon_stream_producer<'closure, 'name: 'closure>(
    stream_map_name: &'name str,
    position: AirPos,
) -> Box<CreateCanonStreamClosure<'closure>> {
    Box::new(move |exec_ctx: &mut ExecutionCtx<'_>, peer_pk: String| -> CanonStream {
        let stream_map = exec_ctx
            .stream_maps
            .get_mut(stream_map_name, position)
            .map(|stream_map| Cow::Borrowed(stream_map))
            .unwrap_or_default();

        let value = stream_map
            .iter_unique_key_object()
            .collect::<serde_json::Map<String, JValue>>();

        let value = ValueAggregate::from_literal_result(LiteralAggregate::new(
            Rc::new(value.into()),
            peer_pk.as_str().into(),
            0.into(),
        ));

        // This single value is a map of StreamMap unique keys to values.
        CanonStream::from_values(vec![value], peer_pk)
    })
}
