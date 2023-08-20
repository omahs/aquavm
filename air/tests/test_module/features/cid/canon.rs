/*
 * Copyright 2022 Fluence Labs Limited
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

use air::ExecutionCidState;
use air::UncatchableError::ValueForCidNotFound;
use air_interpreter_cid::CID;
use air_interpreter_data::{CidStore, CidTracker};
use air_test_framework::AirScriptExecutor;
use air_test_utils::prelude::*;
use pretty_assertions::assert_eq;

#[test]
fn test_canon_ok() {
    let init_peer_id = "init_peer_id";

    let script = format!(
        r#"(seq
       (seq
           (ap 42 $stream)
           (call "{init_peer_id}" ("serv" "func") [] $stream)) ; ok = "to canon"
       (canon "{init_peer_id}" $stream #canon)
    )"#
    );

    let executor =
        AirScriptExecutor::from_annotated(TestRunParameters::from_init_peer_id(init_peer_id), &script).unwrap();
    let result = executor.execute_one(init_peer_id).unwrap();
    let data = data_from_result(&result);

    let mut cid_state = ExecutionCidState::new();

    let stream_exec_state = stream_tracked!(
        "to canon",
        1,
        cid_state,
        peer = init_peer_id,
        service = "serv..0",
        function = "func"
    );

    let service_result_cid = extract_service_result_cid(&stream_exec_state);

    let expected_trace = vec![
        ap(0),
        stream_exec_state,
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": init_peer_id, "service_id": ""},
                "values": [{
                    "result": 42,
                    "tetraplet": {
                        "function_name": "",
                        "json_path": "",
                        "peer_pk": init_peer_id,
                        "service_id": "",
                    },
                    "provenance": Provenance::literal(),
                }, {
                    "result": "to canon",
                    "tetraplet": {
                        "function_name": "func",
                        "json_path": "",
                        "peer_pk": init_peer_id,
                        "service_id": "serv..0",
                    },
                    "provenance": Provenance::service_result(service_result_cid),
                }]
            }),
            &mut cid_state,
        ),
    ];

    assert_eq!(&*data.trace, expected_trace);
    assert_eq!(data.cid_info.value_store, cid_state.value_tracker.into());
    assert_eq!(data.cid_info.tetraplet_store, cid_state.tetraplet_tracker.into());
    assert_eq!(
        data.cid_info.canon_element_store,
        cid_state.canon_element_tracker.into()
    );
    assert_eq!(
        data.cid_info.service_result_store,
        cid_state.service_result_agg_tracker.into()
    );
}

#[test]
fn test_canon_ok_multi() {
    let init_peer_id = "init_peer_id";
    let other_peer_id = "other_peer_id";

    let script = format!(
        r#"(seq
       (seq
           (call "{init_peer_id}" ("serv" "func") [] $stream) ; ok = "to canon"
           (call "{other_peer_id}" ("other_serv" "other_func") [] $stream) ; ok = "other"
       )
       (canon "{init_peer_id}" $stream #canon)
    )"#
    );

    let executor =
        AirScriptExecutor::from_annotated(TestRunParameters::from_init_peer_id(init_peer_id), &script).unwrap();
    let _result1 = executor.execute_one(init_peer_id).unwrap();
    let _result2 = executor.execute_one(other_peer_id).unwrap();
    let result3 = executor.execute_one(init_peer_id).unwrap();
    let data = data_from_result(&result3);

    let mut cid_state = ExecutionCidState::new();

    let stream_state_1 = stream_tracked!(
        "to canon",
        0,
        cid_state,
        peer = init_peer_id,
        service = "serv..0",
        function = "func"
    );
    let service_result_cid_1 = extract_service_result_cid(&stream_state_1);

    let stream_state_2 = stream_tracked!(
        "other",
        1,
        cid_state,
        peer = other_peer_id,
        service = "other_serv..1",
        function = "other_func"
    );
    let service_result_cid_2 = extract_service_result_cid(&stream_state_2);

    let expected_trace = vec![
        stream_state_1,
        stream_state_2,
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": init_peer_id, "service_id": ""},
                "values": [{
                    "result": "to canon",
                    "tetraplet": {
                        "function_name": "func",
                        "json_path": "",
                        "peer_pk": init_peer_id,
                        "service_id": "serv..0",
                    },
                    "provenance": Provenance::service_result(service_result_cid_1),
                }, {
                    "result": "other",
                    "tetraplet": {
                        "function_name": "other_func",
                        "json_path": "",
                        "peer_pk": other_peer_id,
                        "service_id": "other_serv..1",
                    },
                    "provenance": Provenance::service_result(service_result_cid_2),
                }]
            }),
            &mut cid_state,
        ),
    ];

    assert_eq!(&*data.trace, expected_trace);
    assert_eq!(data.cid_info.value_store.len(), 2);
    assert_eq!(data.cid_info.value_store, cid_state.value_tracker.into());
    assert_eq!(data.cid_info.tetraplet_store, cid_state.tetraplet_tracker.into());
    assert_eq!(
        data.cid_info.canon_element_store,
        cid_state.canon_element_tracker.into()
    );
    assert_eq!(
        data.cid_info.service_result_store,
        cid_state.service_result_agg_tracker.into()
    );
}

#[test]
fn test_canon_value_not_found() {
    let init_peer_id = "vm_peer_id";
    let mut vm = create_avm(echo_call_service(), init_peer_id);

    let mut cid_state = ExecutionCidState::new();

    let air_script = format!(
        r#"
       (seq
          (ap 42 $stream)
          (canon "{init_peer_id}" $stream #canon))"#
    );
    let trace = vec![
        ap(0),
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": init_peer_id, "service_id": ""},
                "values": [{
                    "result": 42,
                    "tetraplet": {
                        "function_name": "",
                        "json_path": "",
                        "peer_pk": init_peer_id,
                        "service_id": "",
                    },
                }]
            }),
            &mut cid_state,
        ),
    ];

    let missing_cid = "bagaajyheaiqm6aoj3hpiz4kdhuqsvw4xxzc6z7ap3bufth5ph3shigs6swouetq";
    let value_store: CidStore<_> = cid_state.value_tracker.into();
    assert!(
        value_store.get(&CID::<_>::new(missing_cid)).is_some(),
        "{:#?}",
        value_store
    );

    // Override with fake data.
    cid_state.value_tracker = CidTracker::<_>::new();
    let cur_data = raw_data_from_trace_with_canon(trace, cid_state);
    let result = call_vm!(vm, <_>::default(), air_script, vec![], cur_data);
    let expected_error = ValueForCidNotFound("value", String::from(missing_cid));
    assert!(check_error(&result, expected_error));
}

#[test]
fn test_canon_root_tetraplet_not_found() {
    let init_peer_id = "vm_peer_id";
    let other_peer_id = "other_peer_id";
    let mut vm = create_avm(echo_call_service(), init_peer_id);

    let mut cid_state = ExecutionCidState::new();

    let air_script = format!(
        r#"
       (seq
          (ap 42 $stream)
          (canon "{other_peer_id}" $stream #canon))"#
    );
    let trace = vec![
        ap(0),
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": init_peer_id, "service_id": ""},
                "values": [{
                    "result": 42,
                    "tetraplet": {
                        "function_name": "",
                        "json_path": "",
                        "peer_pk": other_peer_id,
                        "service_id": "",
                    },
                }]
            }),
            &mut cid_state,
        ),
    ];

    let missing_cid = "bagaajyheaiqgpoeo2fka5wbuaetyvbcxu5g66csdtbjwovvws52n7ibqfrpwgwy";
    let tetraplet_store: CidStore<_> = cid_state.tetraplet_tracker.into();
    assert!(
        tetraplet_store.get(&CID::<_>::new(missing_cid)).is_some(),
        "{:#?}",
        tetraplet_store
    );

    let mut fake_tetraplet_tracker = CidTracker::<_>::new();
    fake_tetraplet_tracker
        .track_value(SecurityTetraplet::literal_tetraplet(other_peer_id))
        .unwrap();

    cid_state.tetraplet_tracker = fake_tetraplet_tracker;

    let cur_data = raw_data_from_trace_with_canon(trace, cid_state);
    let result = call_vm!(vm, <_>::default(), air_script, vec![], cur_data);

    let expected_error = ValueForCidNotFound("tetraplet", String::from(missing_cid));
    assert!(check_error(&result, expected_error));
}

#[test]
fn test_canon_tetraplet_not_found() {
    let init_peer_id = "vm_peer_id";
    let mut vm = create_avm(echo_call_service(), init_peer_id);

    let mut cid_state = ExecutionCidState::new();

    let air_script = format!(
        r#"
       (seq
          (call "peer_1" ("serv" "func") [] $stream)
          (canon "{init_peer_id}" $stream #canon))"#
    );
    let trace = vec![
        stream_tracked!(
            42,
            0,
            cid_state,
            peer = "peer_1",
            service = "serv..0",
            function = "func"
        ),
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": init_peer_id, "service_id": ""},
                "values": [{
                    "result": 42,
                    "tetraplet": {
                        "function_name": "func",
                        "json_path": "",
                        "peer_pk": "peer_1",
                        "service_id": "serv",
                    },
                }]
            }),
            &mut cid_state,
        ),
    ];

    let missing_cid = "bagaajyheaiqikwejgtxyvyjx2hcjwprzs25zohusmj4gqevd24mljryjcjntewq";
    let tetraplet_store: CidStore<_> = cid_state.tetraplet_tracker.into();
    assert!(
        tetraplet_store.get(&CID::<_>::new(missing_cid)).is_some(),
        "{:#?}",
        tetraplet_store
    );

    let mut fake_tetraplet_tracker = CidTracker::<_>::new();
    fake_tetraplet_tracker
        .track_value(SecurityTetraplet::literal_tetraplet(init_peer_id))
        .unwrap();

    cid_state.tetraplet_tracker = fake_tetraplet_tracker;
    let cur_data = raw_data_from_trace_with_canon(trace, cid_state);
    let result = call_vm!(vm, <_>::default(), air_script, vec![], cur_data);

    let expected_error = ValueForCidNotFound("tetraplet", String::from(missing_cid));
    assert!(check_error(&result, expected_error), "{}", result.error_message);
}

#[test]
fn test_canon_agg_not_found() {
    let init_peer_id = "vm_peer_id";
    let mut vm = create_avm(echo_call_service(), init_peer_id);

    let mut cid_state = ExecutionCidState::new();

    let air_script = format!(
        r#"
       (seq
          (ap 42 $stream)
          (canon "other_peer_id" $stream #canon))"#
    );
    let trace = vec![
        ap(0),
        canon_tracked(
            json!({
                "tetraplet": {"function_name": "", "json_path": "", "peer_pk": "other_peer_id", "service_id": ""},
                "values": [{
                    "result": 42,
                    "tetraplet": {
                        "function_name": "",
                        "json_path": "",
                        "peer_pk": init_peer_id,
                        "service_id": "",
                    },
                    "provenance": Provenance::literal(),
                }]
            }),
            &mut cid_state,
        ),
    ];

    let missing_cid = "bagaajyheaiqfuixbejqrp3wfx2hr5mj7wavtdn2z4ysy2a77aclmi232jwhhzeq";
    let canon_element_store: CidStore<_> = cid_state.canon_element_tracker.into();
    assert!(
        canon_element_store.get(&CID::<_>::new(missing_cid)).is_some(),
        "{:#?}",
        canon_element_store
    );

    // Fake data
    cid_state.canon_element_tracker = <_>::default();
    let cur_data = raw_data_from_trace_with_canon(trace, cid_state);
    let result = call_vm!(vm, <_>::default(), air_script, vec![], cur_data);

    let expected_error = ValueForCidNotFound("canon aggregate", String::from(missing_cid));
    assert!(check_error(&result, expected_error));
}
