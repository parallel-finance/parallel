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

use cumulus_client_consensus_aura::{AuraConsensus, BuildAuraConsensusParams, SlotProportion};
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};

use polkadot_service::{CollatorPair, ConstructRuntimeApi};
use sc_executor::NativeElseWasmExecutor;
use sc_network_common::service::NetworkBlock;
use sc_service::{Configuration, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};

use primitives::*;

use std::{sync::Arc, time::Duration};

use cumulus_client_cli::CollatorOptions;
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use cumulus_relay_chain_minimal_node::build_minimal_relay_chain_node;

pub use sc_executor::NativeExecutionDispatch;

// Native executor instance.
pub struct ParallelExecutor;
impl sc_executor::NativeExecutionDispatch for ParallelExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        parallel_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        parallel_runtime::native_version()
    }
}

pub struct HeikoExecutor;
impl sc_executor::NativeExecutionDispatch for HeikoExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        heiko_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        heiko_runtime::native_version()
    }
}

pub type FullBackend = sc_service::TFullBackend<Block>;
pub type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;

pub trait IdentifyVariant {
    fn is_parallel(&self) -> bool;

    fn is_heiko(&self) -> bool;

    fn is_vanilla(&self) -> bool;

    fn is_kerria(&self) -> bool;

    fn is_dev(&self) -> bool;
}

impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
    fn is_parallel(&self) -> bool {
        self.id().starts_with("parallel")
    }

    fn is_heiko(&self) -> bool {
        self.id().starts_with("heiko")
    }

    fn is_vanilla(&self) -> bool {
        self.id().starts_with("vanilla")
    }

    fn is_kerria(&self) -> bool {
        self.id().starts_with("kerria")
    }

    fn is_dev(&self) -> bool {
        return self.id().starts_with("vanilla-local-dev");
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
        (
            Option<Telemetry>,
            Option<TelemetryWorkerHandle>,
            Arc<fc_db::Backend<Block>>,
        ),
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

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );
    // FIXME:
    let frontier_backend = crate::rpc::open_frontier_backend(client.clone(), config)?;
    let frontier_block_import =
        FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

    let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

    let import_queue = cumulus_client_consensus_aura::import_queue::<
        sp_consensus_aura::sr25519::AuthorityPair,
        _,
        _,
        _,
        _,
        _,
    >(cumulus_client_consensus_aura::ImportQueueParams {
        block_import: frontier_block_import, // TODO: confirm
        client: client.clone(),
        create_inherent_data_providers: move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *time,
                    slot_duration,
                );

            Ok((slot, time))
        },
        registry: config.prometheus_registry(),
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
        other: (telemetry, telemetry_worker_handle, frontier_backend),
    };

    Ok(params)
}

#[allow(dead_code, unused)]
async fn build_relay_chain_interface(
    polkadot_config: Configuration,
    parachain_config: &Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    task_manager: &mut TaskManager,
    collator_options: CollatorOptions,
) -> RelayChainResult<(
    Arc<(dyn RelayChainInterface + 'static)>,
    Option<CollatorPair>,
)> {
    match collator_options.relay_chain_rpc_url {
        Some(relay_chain_url) => {
            build_minimal_relay_chain_node(polkadot_config, task_manager, relay_chain_url).await
        }
        None => build_inprocess_relay_chain(
            polkadot_config,
            parachain_config,
            telemetry_worker_handle,
            task_manager,
            None,
        ),
    }
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, Executor>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
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
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;
    let (mut telemetry, telemetry_worker_handle, frontier_backend) = params.other;

    let client = params.client.clone();
    let backend = params.backend.clone();

    let mut task_manager = params.task_manager;

    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
    )
    .await
    .map_err(|e| match e {
        RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
        s => s.to_string().into(),
    })?;

    let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);
    let force_authoring = parachain_config.force_authoring;
    let validator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);

    let (network, system_rpc_tx, tx_handler_controller, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: import_queue.clone(),
            block_announce_validator_builder: Some(Box::new(|_| {
                Box::new(block_announce_validator)
            })),
            warp_sync: None,
        })?;

    // TODO: check and confirm
    let filter_pool: FilterPool = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
    let fee_history_cache: FeeHistoryCache = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
    let overrides = crate::rpc::overrides_handle(client.clone());

    // Frontier offchain DB task. Essential.
    // Maps emulated ethereum data to substrate native data.
    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        fc_mapping_sync::MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend.clone(),
            frontier_backend.clone(),
            3,
            0,
            fc_mapping_sync::SyncStrategy::Parachain,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Frontier `EthFilterApi` maintenance. Manages the pool of user-created Filters.
    // Each filter is allowed to stay in the pool for 100 blocks.
    const FILTER_RETAIN_THRESHOLD: u64 = 100;
    task_manager.spawn_essential_handle().spawn(
        "frontier-filter-pool",
        Some("frontier"),
        fc_rpc::EthTask::filter_pool_task(
            client.clone(),
            filter_pool.clone(),
            FILTER_RETAIN_THRESHOLD,
        ),
    );

    const FEE_HISTORY_LIMIT: u64 = 2048;
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        fc_rpc::EthTask::fee_history_task(
            client.clone(),
            overrides.clone(),
            fee_history_cache.clone(),
            FEE_HISTORY_LIMIT,
        ),
    );

    let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
        task_manager.spawn_handle(),
        overrides.clone(),
        50,
        50,
        prometheus_registry.clone(),
    ));

    let rpc_builder = {
        // let client = client.clone();
        // let pool = transaction_pool.clone();

        // Box::new(move |deny_unsafe, _| {
        //     let deps = crate::rpc::FullDeps {
        //         client: client.clone(),
        //         pool: pool.clone(),
        //         deny_unsafe,
        //     };

        //     crate::rpc::create_full(deps).map_err(Into::into)
        // })
        let client = client.clone();
        let network = network.clone();
        let transaction_pool = transaction_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let overrides = overrides.clone();
        let fee_history_cache = fee_history_cache.clone();
        let block_data_cache = block_data_cache.clone();

        Box::new(move |deny_unsafe, subscription| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                graph: transaction_pool.pool().clone(),
                network: network.clone(),
                is_authority,
                deny_unsafe,
                frontier_backend: frontier_backend.clone(),
                filter_pool: filter_pool.clone(),
                fee_history_limit: FEE_HISTORY_LIMIT,
                fee_history_cache: fee_history_cache.clone(),
                block_data_cache: block_data_cache.clone(),
                overrides: overrides.clone(),
            };

            crate::rpc::create_full(deps, subscription).map_err(Into::into)
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
        network: network.clone(),
        client: client.clone(),
        keystore: params.keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: Box::new(rpc_builder),
        backend: backend.clone(),
        system_rpc_tx,
        tx_handler_controller,
        config: parachain_config,
        telemetry: telemetry.as_mut(),
    })?;

    let announce_block = {
        let network = network.clone();
        Arc::new(move |hash, data| network.announce_block(hash, data))
    };

    let relay_chain_slot_duration = Duration::from_secs(6);

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

        let relay_chain_for_aura = relay_chain_interface.clone();
        let parachain_consensus =
            AuraConsensus::build::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _>(
                BuildAuraConsensusParams {
                    proposer_factory,
                    create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
                        let relay_chain_interface = relay_chain_for_aura.clone();
                        async move {
                            let parachain_inherent =
                            cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
                                relay_parent,
                                &relay_chain_interface,
                                &validation_data,
                                id,
                            )
                            .await;

                            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                            let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

                            let parachain_inherent = parachain_inherent.ok_or_else(|| {
                                Box::<dyn std::error::Error + Send + Sync>::from(
                                    "Failed to create parachain inherent",
                                )
                            })?;

                            Ok((slot, timestamp, parachain_inherent))
                        }
                    },
                    block_import: client.clone(),
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
                },
            );

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_interface,
            spawner,
            parachain_consensus,
            import_queue,
            collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
            relay_chain_slot_duration,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_interface,
            relay_chain_slot_duration,
            import_queue,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

/// Start a normal parachain node.
#[allow(dead_code, unused)]
pub async fn start_node<RuntimeApi, Executor>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
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
    start_node_impl(parachain_config, polkadot_config, collator_options, id).await
}

/// Build a partial chain component config
pub fn new_dev_partial<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (
            FrontierBlockImport<
                Block,
                Arc<FullClient<RuntimeApi, Executor>>,
                FullClient<RuntimeApi, Executor>,
            >,
            Option<Telemetry>,
            Arc<fc_db::Backend<Block>>,
        ),
    >,
    ServiceError,
>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + sp_session::SessionKeys<Block>
        + sp_api::ApiExt<
            Block,
            StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
        > + sp_offchain::OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_consensus_aura::AuraApi<Block, AuraId>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>
        + orml_oracle_rpc::OracleRuntimeApi<Block, DataProviderId, CurrencyId, TimeStampedPrice>
        + pallet_loans_rpc::LoansRuntimeApi<Block, AccountId, Balance>
        + pallet_router_rpc::RouterRuntimeApi<Block, Balance>,
    sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    if config.keystore_remote.is_some() {
        return Err(ServiceError::Other(
            "Remote Keystores are not supported.".to_string(),
        ));
    }

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
    let executor = sc_executor::NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);
    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });
    let select_chain = sc_consensus::LongestChain::new(backend.clone());
    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );
    let frontier_backend = crate::rpc::open_frontier_backend(client.clone(), config)?;

    let frontier_block_import =
        FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(frontier_block_import.clone()),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (frontier_block_import, telemetry, frontier_backend),
    })
}

/// Builds a new service.
pub fn start_dev_node<RuntimeApi, Executor>(
    config: Configuration,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + sp_session::SessionKeys<Block>
        + sp_api::ApiExt<
            Block,
            StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
        > + sp_offchain::OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_consensus_aura::AuraApi<Block, AuraId>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>
        + orml_oracle_rpc::OracleRuntimeApi<Block, DataProviderId, CurrencyId, TimeStampedPrice>
        + pallet_loans_rpc::LoansRuntimeApi<Block, AccountId, Balance>
        + pallet_router_rpc::RouterRuntimeApi<Block, Balance>,
    sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, mut telemetry, frontier_backend),
    } = new_dev_partial::<RuntimeApi, Executor>(&config)?;

    let (network, system_rpc_tx, tx_handler_controller, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: None,
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let filter_pool: FilterPool = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
    let fee_history_cache: FeeHistoryCache = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
    let overrides = crate::rpc::overrides_handle(client.clone());

    // Frontier offchain DB task. Essential.
    // Maps emulated ethereum data to substrate native data.
    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        fc_mapping_sync::MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend.clone(),
            frontier_backend.clone(),
            3,
            0,
            fc_mapping_sync::SyncStrategy::Parachain,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Frontier `EthFilterApi` maintenance. Manages the pool of user-created Filters.
    // Each filter is allowed to stay in the pool for 100 blocks.
    const FILTER_RETAIN_THRESHOLD: u64 = 100;
    task_manager.spawn_essential_handle().spawn(
        "frontier-filter-pool",
        Some("frontier"),
        fc_rpc::EthTask::filter_pool_task(
            client.clone(),
            filter_pool.clone(),
            FILTER_RETAIN_THRESHOLD,
        ),
    );

    const FEE_HISTORY_LIMIT: u64 = 2048;
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        fc_rpc::EthTask::fee_history_task(
            client.clone(),
            overrides.clone(),
            fee_history_cache.clone(),
            FEE_HISTORY_LIMIT,
        ),
    );

    let role = config.role.clone();
    let prometheus_registry = config.prometheus_registry().cloned();
    let is_authority = config.role.is_authority();

    let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
        task_manager.spawn_handle(),
        overrides.clone(),
        50,
        50,
        prometheus_registry.clone(),
    ));

    let rpc_extensions_builder = {
        let client = client.clone();
        let network = network.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, subscription| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                graph: transaction_pool.pool().clone(),
                network: network.clone(),
                is_authority,
                deny_unsafe,
                frontier_backend: frontier_backend.clone(),
                filter_pool: filter_pool.clone(),
                fee_history_limit: FEE_HISTORY_LIMIT,
                fee_history_cache: fee_history_cache.clone(),
                block_data_cache: block_data_cache.clone(),
                overrides: overrides.clone(),
            };

            let io = crate::rpc::create_full(deps, subscription)
                .map_err::<ServiceError, _>(Into::into)?;

            Ok(io)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        config,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    if role.is_authority() {
        let env = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let target_gas_price = 1;
        let commands_stream = transaction_pool
            .pool()
            .clone()
            .validated_pool()
            .import_notification_stream()
            .map(
                |_| sc_consensus_manual_seal::rpc::EngineCommand::SealNewBlock {
                    create_empty: true,
                    finalize: true,
                    parent_hash: None,
                    sender: None,
                },
            );
        let client_for_cidp = client.clone();

        // Background authorship future
        let authorship_future = manual_seal::run_manual_seal(manual_seal::ManualSealParams {
            block_import,
            env,
            client: client.clone(),
            pool: transaction_pool.clone(),
            commands_stream,
            select_chain,
            consensus_data_provider: None,
            create_inherent_data_providers: move |block: Hash, _| {
                let current_para_block = client_for_cidp
                    .number(block)
                    .expect("Header lookup should succeed")
                    .expect("Header passed in as parent should be present in backend.");

                async move {
                    let dynamic_fee =
                        fp_dynamic_fee::InherentDataProvider(sp_core::U256::from(target_gas_price));

                    let mocked_parachain = MockValidationDataInherentDataProvider {
                        current_para_block,
                        relay_offset: 1000,
                        relay_blocks_per_para_block: 2,
                        para_blocks_per_relay_epoch: 10,
                        relay_randomness_config: (),
                        xcm_config: Default::default(),
                        raw_downward_messages: vec![],
                        raw_horizontal_messages: vec![],
                    };

                    Ok((
                        sp_timestamp::InherentDataProvider::from_system_time(),
                        mocked_parachain,
                        dynamic_fee,
                    ))
                }
            },
        });
        // we spawn the future on a background thread managed by service.
        task_manager.spawn_essential_handle().spawn_blocking(
            "instant-seal",
            None,
            authorship_future,
        );
    }
    log::info!("Manual Seal Ready");

    network_starter.start_network();
    Ok(task_manager)
}
