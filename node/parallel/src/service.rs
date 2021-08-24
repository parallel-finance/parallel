// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::client::Block;
use cumulus_client_consensus_aura::{
    build_aura_consensus, BuildAuraConsensusParams, SlotProportion,
};
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;

use polkadot_service::ConstructRuntimeApi;
use sc_client_api::call_executor::ExecutorProvider;
use sc_executor::native_executor_instance;
use sc_service::{Configuration, PartialComponents, Role, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};

use primitives::*;
use sp_consensus::SlotData;

use std::sync::Arc;

pub use sc_executor::{NativeExecutionDispatch, NativeExecutor};

// Native executor instance.
native_executor_instance!(
    pub ParallelExecutor,
    parallel_runtime::api::dispatch,
    parallel_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

native_executor_instance!(
    pub HeikoExecutor,
    heiko_runtime::api::dispatch,
    heiko_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

pub type FullBackend = sc_service::TFullBackend<Block>;
pub type FullClient<RuntimeApi, Executor> = sc_service::TFullClient<Block, RuntimeApi, Executor>;

pub trait IdentifyVariant {
    fn is_parallel(&self) -> bool;

    fn is_heiko(&self) -> bool;
}

impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
    fn is_parallel(&self) -> bool {
        self.id().starts_with("parallel")
    }

    fn is_heiko(&self) -> bool {
        self.id().starts_with("heiko")
    }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<
    PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        (),
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (Option<Telemetry>, Option<TelemetryWorkerHandle>),
    >,
    sc_service::Error,
>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: crate::client::RuntimeApiCollection<
        StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
    >,
    Executor: NativeExecutionDispatch + 'static,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );
    let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

    let import_queue = cumulus_client_consensus_aura::import_queue::<
        sp_consensus_aura::sr25519::AuthorityPair,
        _,
        _,
        _,
        _,
        _,
        _,
    >(cumulus_client_consensus_aura::ImportQueueParams {
        block_import: client.clone(),
        client: client.clone(),
        create_inherent_data_providers: move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                    *time,
                    slot_duration.slot_duration(),
                );

            Ok((time, slot))
        },
        registry: config.prometheus_registry().clone(),
        can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
        spawner: &task_manager.spawn_essential_handle(),
        telemetry: telemetry.as_ref().map(|telemetry| telemetry.handle()),
    })?;

    let params = PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (telemetry, telemetry_worker_handle),
    };

    Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, Executor>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    id: ParaId,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: crate::client::RuntimeApiCollection<
        StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
    >,
    Executor: NativeExecutionDispatch + 'static,
{
    if matches!(parachain_config.role, Role::Light) {
        return Err("Light client not supported!".into());
    }

    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;
    let (mut telemetry, telemetry_worker_handle) = params.other;

    let relay_chain_full_node =
        cumulus_client_service::build_polkadot_full_node(polkadot_config, telemetry_worker_handle)
            .map_err(|e| match e {
                polkadot_service::Error::Sub(x) => x,
                s => format!("{}", s).into(),
            })?;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let block_announce_validator = build_block_announce_validator(
        relay_chain_full_node.client.clone(),
        id,
        Box::new(relay_chain_full_node.network.clone()),
        relay_chain_full_node.backend.clone(),
    );
    let force_authoring = parachain_config.force_authoring;
    let validator = parachain_config.role.is_authority();

    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let mut task_manager = params.task_manager;
    let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);

    let (network, system_rpc_tx, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: import_queue.clone(),
            on_demand: None,
            block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
            warp_sync: None,
        })?;

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
            };

            crate::rpc::create_full(deps)
        })
    };

    if parachain_config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &parachain_config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        on_demand: None,
        remote_blockchain: None,
        rpc_extensions_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.sync_keystore(),
        backend: backend.clone(),
        network: network.clone(),
        system_rpc_tx,
        telemetry: telemetry.as_mut(),
    })?;

    let announce_block = {
        let network = network.clone();
        Arc::new(move |hash, data| network.announce_block(hash, data))
    };

    if validator {
        let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );
        let spawner = task_manager.spawn_handle();
        let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

        let relay_chain_backend = relay_chain_full_node.backend.clone();
        let relay_chain_client = relay_chain_full_node.client.clone();

        let parachain_consensus = build_aura_consensus::<
            sp_consensus_aura::sr25519::AuthorityPair,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >(BuildAuraConsensusParams {
            proposer_factory,
            create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
                let parachain_inherent =
					cumulus_primitives_parachain_inherent::ParachainInherentData::create_at_with_client(
						relay_parent,
						&relay_chain_client,
						&*relay_chain_backend,
						&validation_data,
						id,
					);
                async move {
                    let time = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
							*time,
							slot_duration.slot_duration(),
						);

                    let parachain_inherent = parachain_inherent.ok_or_else(|| {
                        Box::<dyn std::error::Error + Send + Sync>::from(
                            "Failed to create parachain inherent",
                        )
                    })?;
                    Ok((time, slot, parachain_inherent))
                }
            },
            block_import: client.clone(),
            relay_chain_client: relay_chain_full_node.client.clone(),
            relay_chain_backend: relay_chain_full_node.backend.clone(),
            para_client: client.clone(),
            backoff_authoring_blocks: Option::<()>::None,
            sync_oracle: network,
            keystore: params.keystore_container.sync_keystore(),
            force_authoring,
            slot_duration,
            // We got around 500ms for proposing
            block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
            // And a maximum of 750ms if slots are skipped
            max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
            telemetry: telemetry.as_ref().map(|telemetry| telemetry.handle()),
        });

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_full_node,
            spawner,
            parachain_consensus,
            import_queue,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_full_node,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

/// Start a normal parachain node.
pub async fn start_node<RuntimeApi, Executor>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    id: ParaId,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: crate::client::RuntimeApiCollection<
        StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
    >,
    RuntimeApi::RuntimeApi: sp_consensus_aura::AuraApi<Block, AuraId>,
    Executor: NativeExecutionDispatch + 'static,
{
    start_node_impl(parachain_config, polkadot_config, id).await
}
