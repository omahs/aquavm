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

use crate::ExecutionError;

// TODO rename to SigningTracker
use air_interpreter_signatures::{FullSignatureStore, PeerCidTracker};

#[cfg(feature = "sign_cids")]
pub(crate) fn sign_produced_cids(
    signature_tracker: &mut PeerCidTracker,
    signature_store: &mut FullSignatureStore,
    keypair: &fluence_keypair::KeyPair,
) -> Result<(), ExecutionError> {
    use crate::UncatchableError;
    use air_interpreter_signatures::CidTracker as _;

    let signature = signature_tracker
        .gen_signature(keypair)
        .map_err(UncatchableError::SigningError)?;
    let public_key = keypair.public().into();
    signature_store.put(public_key, signature);
    Ok(())
}

#[cfg(not(feature = "sign_cids"))]
pub(crate) fn sign_produced_cids(
    _signature_tracker: &mut PeerCidTracker,
    _signature_store: &mut FullSignatureStore,
    _keypair: &fluence_keypair::KeyPair,
) -> Result<(), ExecutionError> {
    Ok(())
}