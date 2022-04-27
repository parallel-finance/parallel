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

use std::path::PathBuf;

use clap::Parser;
use sc_cli::{KeySubcommand, SignCmd, VanityCmd, VerifyCmd};

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management CLI utilities
    #[clap(subcommand)]
    Key(KeySubcommand),

    Sign(SignCmd),

    Vanity(VanityCmd),

    Verify(VerifyCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Export the genesis state of the parachain.
    #[clap(name = "export-genesis-state")]
    ExportGenesisState(ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[clap(name = "export-genesis-wasm")]
    ExportGenesisWasm(ExportGenesisWasmCommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, Parser)]
pub struct ExportGenesisStateCommand {
    /// Output file name or stdout if unspecified.
    #[clap(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Write output in binary. Default is to write in hex.
    #[clap(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis state should be exported.
    #[clap(long)]
    pub chain: Option<String>,
}

/// Command for exporting the genesis wasm file.
#[derive(Debug, Parser)]
pub struct ExportGenesisWasmCommand {
    /// Output file name or stdout if unspecified.
    #[clap(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Write output in binary. Default is to write in hex.
    #[clap(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis wasm file should be exported.
    #[clap(long)]
    pub chain: Option<String>,
}

#[derive(Debug, Parser)]
#[clap(
    propagate_version = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: cumulus_client_cli::RunCmd,

    /// Relaychain arguments
    #[clap(raw = true)]
    pub relaychain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
    /// Create a new instance of `Self`.
    pub fn new<'a>(
        base_path: Option<PathBuf>,
        chain_id: Option<String>,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        Self {
            base_path,
            chain_id,
            base: polkadot_cli::RunCmd::parse_from(relay_chain_args),
        }
    }
}
