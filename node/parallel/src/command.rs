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

use crate::{
    chain_spec::{self, set_default_ss58_version},
    cli::{Cli, RelayChainCli, Subcommand},
    service::IdentifyVariant,
};
use codec::Encode;
use frame_benchmarking_cli::BenchmarkCmd;
use log::info;
use primitives::ParaId;
use sc_cli::{
    ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
    NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::{
    config::{BasePath, PrometheusConfig},
    PartialComponents,
};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};

use std::net::SocketAddr;

const CHAIN_NAME: &str = "Parallel";
const PARALLEL_PARA_ID: u32 = 2012;
const HEIKO_PARA_ID: u32 = 2085;

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match id {
        "dev" => Box::new(chain_spec::vanilla::local_development_config(ParaId::from(
            HEIKO_PARA_ID,
        ))),
        "heiko-dev" => Box::new(chain_spec::heiko::heiko_dev_config(ParaId::from(
            HEIKO_PARA_ID,
        ))),
        "heiko" => Box::new(chain_spec::heiko::heiko_config(ParaId::from(
            HEIKO_PARA_ID,
        ))?),
        "parallel-dev" => Box::new(chain_spec::parallel::parallel_dev_config(ParaId::from(
            PARALLEL_PARA_ID,
        ))),
        "" | "parallel" => Box::new(chain_spec::parallel::parallel_config(ParaId::from(
            PARALLEL_PARA_ID,
        ))?),
        "vanilla-dev" => Box::new(chain_spec::vanilla::vanilla_dev_config(ParaId::from(
            HEIKO_PARA_ID,
        ))),
        "kerria-dev" => Box::new(chain_spec::kerria::kerria_dev_config(ParaId::from(
            PARALLEL_PARA_ID,
        ))),
        path => {
            let path = std::path::PathBuf::from(path);
            let starts_with = |prefix: &str| {
                path.file_name()
                    .map(|f| f.to_str().map(|s| s.starts_with(&prefix)))
                    .flatten()
                    .unwrap_or(false)
            };

            if starts_with("parallel") {
                Box::new(chain_spec::parallel::ChainSpec::from_json_file(path)?)
            } else if starts_with("heiko") {
                Box::new(chain_spec::heiko::ChainSpec::from_json_file(path)?)
            } else if starts_with("vanilla") {
                Box::new(chain_spec::vanilla::ChainSpec::from_json_file(path)?)
            } else if starts_with("kerria") {
                Box::new(chain_spec::kerria::ChainSpec::from_json_file(path)?)
            } else {
                return Err(
                    "chain_spec's filename must start with parallel/heiko/kerria/vanilla".into(),
                );
            }
        }
    })
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        format!("{} Node", CHAIN_NAME)
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/parallel-finance/parallel/issues".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        load_spec(id)
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        if chain_spec.is_parallel() {
            &parallel_runtime::VERSION
        } else if chain_spec.is_heiko() {
            &heiko_runtime::VERSION
        } else if chain_spec.is_vanilla() || chain_spec.is_dev() {
            &vanilla_runtime::VERSION
        } else if chain_spec.is_kerria() {
            &kerria_runtime::VERSION
        } else {
            unreachable!()
        }
    }
}

impl SubstrateCli for RelayChainCli {
    fn impl_name() -> String {
        format!("{} Parachain Collator", CHAIN_NAME)
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        format!(
            "{} parachain collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		rococo-collator [parachain-args] -- [relaychain-args]",
            CHAIN_NAME
        )
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/parallel-finance/parallel/issues".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        polkadot_cli::Cli::native_runtime_version(chain_spec)
    }
}

macro_rules! switch_runtime {
    ($chain_spec:expr, { $( $code:tt )* }) => {
        if $chain_spec.is_parallel() {
            #[allow(unused_imports)]
            use crate::service::ParallelExecutor as Executor;
            #[allow(unused_imports)]
            use parallel_runtime::{RuntimeApi, Block};

            $( $code )*
        } else if $chain_spec.is_heiko() {
            #[allow(unused_imports)]
            use crate::service::HeikoExecutor as Executor;
            #[allow(unused_imports)]
            use heiko_runtime::{RuntimeApi, Block};

            $( $code )*
        } else if ($chain_spec.is_vanilla() || $chain_spec.is_dev()) {
            #[allow(unused_imports)]
            use crate::service::VanillaExecutor as Executor;
            #[allow(unused_imports)]
            use vanilla_runtime::{RuntimeApi, Block};

            $( $code )*
        } else if $chain_spec.is_kerria() {
            #[allow(unused_imports)]
            use crate::service::KerriaExecutor as Executor;
            #[allow(unused_imports)]
            use kerria_runtime::{RuntimeApi, Block};

            $( $code )*
        } else {
            unreachable!();
        }
    };
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                    Ok((cmd.run(client, config.database), task_manager))
                })
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                    Ok((cmd.run(client, config.chain_spec), task_manager))
                })
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
                let relay_chain_id = extension.map(|e| e.relay_chain.clone());

                let polkadot_cli = RelayChainCli::new(
                    config.base_path.as_ref().map(|x| x.path().join("polkadot")),
                    relay_chain_id,
                    [RelayChainCli::executable_name()]
                        .iter()
                        .chain(cli.relaychain_args.iter()),
                );

                let polkadot_config = SubstrateCli::create_configuration(
                    &polkadot_cli,
                    &polkadot_cli,
                    config.tokio_handle.clone(),
                )
                .map_err(|err| format!("Relay chain argument error: {}", err))?;

                cmd.run(config, polkadot_config)
            })
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let PartialComponents {
                        client,
                        task_manager,
                        backend,
                        ..
                    } = crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                    Ok((cmd.run(client, backend, None), task_manager))
                })
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);
            switch_runtime!(chain_spec, {
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if cfg!(feature = "runtime-benchmarks") {
                            runner.sync_run(|config| cmd.run::<Block, Executor>(config))
                        } else {
                            Err("Benchmarking wasn't enabled when building the node. \
						You can enable it with `--features runtime-benchmarks`."
                                .into())
                        }
                    }
                    BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
                        let partials =
                            crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                        cmd.run(partials.client)
                    }),
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Storage(_) => Err(
                        "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                            .into(),
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
                        let partials =
                            crate::service::new_partial::<RuntimeApi, Executor>(&config)?;
                        let db = partials.backend.expose_db();
                        let storage = partials.backend.expose_storage();

                        cmd.run(config, partials.client.clone(), db, storage)
                    }),
                    //todo:implement following benchmark sub command
                    BenchmarkCmd::Overhead(_)
                    | BenchmarkCmd::Machine(_)
                    | BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
                }
            })
        }
        Some(Subcommand::ExportGenesisState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            switch_runtime!(chain_spec, {
                return runner.sync_run(|_config| {
                    let spec =
                        cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
                    let state_version = Cli::native_runtime_version(&spec).state_version();
                    cmd.run::<Block>(&*spec, state_version)
                });
            })
        }
        Some(Subcommand::ExportGenesisWasm(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|_config| {
                let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
                cmd.run(&*spec)
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
            type HostFunctionsOf<E> = ExtendedHostFunctions<
                sp_io::SubstrateHostFunctions,
                <E as NativeExecutionDispatch>::ExtendHostFunctions,
            >;

            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.async_run(|config| {
                    let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                    let task_manager =
                        sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                            .map_err(|e| {
                                sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                            })?;

                    Ok((cmd.run::<Block, HostFunctionsOf<Executor>>(), task_manager))
                })
            })
        }
        None => {
            let runner = cli.create_runner(&cli.run.normalize())?;
            let collator_options = cli.run.collator_options();
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec);

            switch_runtime!(chain_spec, {
                runner.run_node_until_exit(|config| async move {
                    if config.chain_spec.is_dev() {
                        return crate::service::start_dev_node::<RuntimeApi, Executor>(config).map_err(Into::into);
                    }
                    let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
                    let relay_chain_id = extension.map(|e| e.relay_chain.clone());
                    let para_chain_id = extension
                        .map(|e| e.para_id)
                        .ok_or("The para_id is required in chain-spec.")?;
                    info!("Relaychain Chain Id: {:?}", relay_chain_id);

                    let polkadot_cli = RelayChainCli::new(
                        config.base_path.as_ref().map(|x| x.path().join("polkadot")),
                        relay_chain_id,
                        [RelayChainCli::executable_name()]
                            .iter()
                            .chain(cli.relaychain_args.iter()),
                    );

                    info!("Relaychain Args: {}", cli.relaychain_args.join(" "));

                    let id = ParaId::from(para_chain_id);

                    let parachain_account =
                        AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(
                            &id,
                        );
                    let state_version =
                        RelayChainCli::native_runtime_version(&config.chain_spec).state_version();

                    let block: Block = cumulus_client_cli::generate_genesis_block(config.chain_spec.as_ref(), state_version)
                        .map_err(|e| format!("{:?}", e))?;
                    let genesis_state =
                        format!("0x{:?}", HexDisplay::from(&block.header().encode()));

                    let polkadot_config = SubstrateCli::create_configuration(
                        &polkadot_cli,
                        &polkadot_cli,
                        config.tokio_handle.clone(),
                    )
                    .map_err(|err| format!("Relay chain argument error: {}", err))?;

                    info!("Parachain id: {:?}", id);
                    info!("Parachain Account: {}", parachain_account);
                    info!("Parachain genesis state: {}", genesis_state);
                    info!(
                        "Is collating: {}",
                        if config.role.is_authority() {
                            "yes"
                        } else {
                            "no"
                        }
                    );

                    crate::service::start_node::<RuntimeApi, Executor>(
                        config,
                        polkadot_config,
                        collator_options,
                        id,
                    )
                    .await
                    .map(|r| r.0)
                    .map_err(Into::into)
                })
            })
        }
    }
}

impl DefaultConfigurationValues for RelayChainCli {
    fn p2p_listen_port() -> u16 {
        30334
    }

    fn rpc_ws_listen_port() -> u16 {
        9945
    }

    fn rpc_http_listen_port() -> u16 {
        9934
    }

    fn prometheus_listen_port() -> u16 {
        9616
    }
}

impl CliConfiguration<Self> for RelayChainCli {
    fn shared_params(&self) -> &SharedParams {
        self.base.base.shared_params()
    }

    fn import_params(&self) -> Option<&ImportParams> {
        self.base.base.import_params()
    }

    fn network_params(&self) -> Option<&NetworkParams> {
        self.base.base.network_params()
    }

    fn keystore_params(&self) -> Option<&KeystoreParams> {
        self.base.base.keystore_params()
    }

    fn base_path(&self) -> Result<Option<BasePath>> {
        Ok(self
            .shared_params()
            .base_path()?
            .or_else(|| self.base_path.clone().map(Into::into)))
    }

    fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.base.base.rpc_http(default_listen_port)
    }

    fn rpc_ipc(&self) -> Result<Option<String>> {
        self.base.base.rpc_ipc()
    }

    fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.base.base.rpc_ws(default_listen_port)
    }

    fn prometheus_config(
        &self,
        default_listen_port: u16,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<PrometheusConfig>> {
        self.base
            .base
            .prometheus_config(default_listen_port, chain_spec)
    }

    fn init<F>(
        &self,
        _support_url: &String,
        _impl_version: &String,
        _logger_hook: F,
        _config: &sc_service::Configuration,
    ) -> Result<()>
    where
        F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
    {
        unreachable!("PolkadotCli is never initialized; qed");
    }

    fn chain_id(&self, is_dev: bool) -> Result<String> {
        let chain_id = self.base.base.chain_id(is_dev)?;

        Ok(if chain_id.is_empty() {
            self.chain_id.clone().unwrap_or_default()
        } else {
            chain_id
        })
    }

    fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
        self.base.base.role(is_dev)
    }

    fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
        self.base.base.transaction_pool(is_dev)
    }

    fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
        self.base.base.trie_cache_maximum_size()
    }

    fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
        self.base.base.rpc_methods()
    }

    fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
        self.base.base.rpc_ws_max_connections()
    }

    fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
        self.base.base.rpc_cors(is_dev)
    }

    fn default_heap_pages(&self) -> Result<Option<u64>> {
        self.base.base.default_heap_pages()
    }

    fn force_authoring(&self) -> Result<bool> {
        self.base.base.force_authoring()
    }

    fn disable_grandpa(&self) -> Result<bool> {
        self.base.base.disable_grandpa()
    }

    fn max_runtime_instances(&self) -> Result<Option<usize>> {
        self.base.base.max_runtime_instances()
    }

    fn announce_block(&self) -> Result<bool> {
        self.base.base.announce_block()
    }

    fn telemetry_endpoints(
        &self,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
        self.base.base.telemetry_endpoints(chain_spec)
    }

    fn node_name(&self) -> Result<String> {
        self.base.base.node_name()
    }
}
