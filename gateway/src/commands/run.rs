// Copyright 2020-2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::commands::helpers::{
    ensure_config_version_compatibility, OverrideConfig, OverrideNetworkRequesterConfig,
};
use crate::node::helpers::node_details;
use crate::support::config::build_config;
use clap::Args;
use nym_bin_common::output_format::OutputFormat;
use nym_config::helpers::SPECIAL_ADDRESSES;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct Run {
    /// Id of the gateway we want to run
    #[clap(long)]
    id: String,

    /// The custom host on which the gateway will be running for receiving sphinx packets
    #[clap(long)]
    host: Option<IpAddr>,

    /// The port on which the gateway will be listening for sphinx packets
    #[clap(long)]
    mix_port: Option<u16>,

    /// The port on which the gateway will be listening for clients gateway-requests
    #[clap(long)]
    clients_port: Option<u16>,

    /// Path to sqlite database containing all gateway persistent data
    #[clap(long)]
    datastore: Option<PathBuf>,

    /// Comma separated list of endpoints of nym APIs
    #[clap(
        long,
        alias = "validator_apis",
        value_delimiter = ',',
        group = "network"
    )]
    // the alias here is included for backwards compatibility (1.1.4 and before)
    nym_apis: Option<Vec<url::Url>>,

    /// Comma separated list of endpoints of the validator
    #[clap(
        long,
        alias = "validators",
        alias = "nyxd_validators",
        value_delimiter = ',',
        hide = true
    )]
    // the alias here is included for backwards compatibility (1.1.4 and before)
    nyxd_urls: Option<Vec<url::Url>>,

    /// Cosmos wallet mnemonic
    #[clap(long)]
    mnemonic: Option<bip39::Mnemonic>,

    /// Set this gateway to work only with coconut credentials; that would disallow clients to
    /// bypass bandwidth credential requirement
    #[clap(long, hide = true)]
    only_coconut_credentials: Option<bool>,

    /// Enable/disable gateway anonymized statistics that get sent to a statistics aggregator server
    #[clap(long)]
    enabled_statistics: Option<bool>,

    /// URL where a statistics aggregator is running. The default value is a Nym aggregator server
    #[clap(long)]
    statistics_service_url: Option<url::Url>,

    /// Allows this gateway to run an embedded network requester for minimal network overhead
    #[clap(long)]
    with_network_requester: Option<bool>,

    // ##### NETWORK REQUESTER FLAGS #####
    /// Specifies whether this network requester should run in 'open-proxy' mode
    #[arg(long)]
    open_proxy: Option<bool>,

    /// Enable service anonymized statistics that get sent to a statistics aggregator server
    #[arg(long)]
    enable_statistics: Option<bool>,

    /// Mixnet client address where a statistics aggregator is running. The default value is a Nym
    /// aggregator client
    #[arg(long)]
    statistics_recipient: Option<String>,

    /// Mostly debug-related option to increase default traffic rate so that you would not need to
    /// modify config post init
    #[arg(long, hide = true, conflicts_with = "medium_toggle")]
    fastmode: bool,

    /// Disable loop cover traffic and the Poisson rate limiter (for debugging only)
    #[arg(long, hide = true, conflicts_with = "medium_toggle")]
    no_cover: bool,

    /// Enable medium mixnet traffic, for experiments only.
    /// This includes things like disabling cover traffic, no per hop delays, etc.
    #[arg(
        long,
        hide = true,
        conflicts_with = "no_cover",
        conflicts_with = "fastmode"
    )]
    medium_toggle: bool,

    /// Path to .json file containing custom network specification.
    /// Only usable when local network requester is enabled.
    #[clap(long, group = "network", hide = true)]
    custom_mixnet: Option<PathBuf>,

    #[clap(short, long, default_value_t = OutputFormat::default())]
    output: OutputFormat,
}

impl From<Run> for OverrideConfig {
    fn from(run_config: Run) -> Self {
        OverrideConfig {
            host: run_config.host,
            mix_port: run_config.mix_port,
            clients_port: run_config.clients_port,
            datastore: run_config.datastore,
            nym_apis: run_config.nym_apis,
            mnemonic: run_config.mnemonic,

            enabled_statistics: run_config.enabled_statistics,
            statistics_service_url: run_config.statistics_service_url,
            nyxd_urls: run_config.nyxd_urls,
            only_coconut_credentials: run_config.only_coconut_credentials,
            with_network_requester: run_config.with_network_requester,
        }
    }
}

impl<'a> From<&'a Run> for OverrideNetworkRequesterConfig {
    fn from(value: &'a Run) -> Self {
        OverrideNetworkRequesterConfig {
            fastmode: value.fastmode,
            no_cover: value.no_cover,
            medium_toggle: value.medium_toggle,
            open_proxy: value.open_proxy,
            enable_statistics: value.enable_statistics,
            statistics_recipient: value.statistics_recipient.clone(),
        }
    }
}

fn show_binding_warning(address: &str) {
    eprintln!("\n##### NOTE #####");
    eprintln!(
        "\nYou are trying to bind to {address} - you might not be accessible to other nodes\n\
         You can ignore this warning if you're running setup on a local network \n\
         or have used different host when bonding your node"
    );
    eprintln!("\n\n");
}

pub async fn execute(args: Run) -> anyhow::Result<()> {
    let id = args.id.clone();
    eprintln!("Starting gateway {id}...");

    let output = args.output;
    let custom_mixnet = args.custom_mixnet.clone();
    let nr_opts = (&args).into();

    let config = build_config(id, args)?;
    ensure_config_version_compatibility(&config)?;

    if SPECIAL_ADDRESSES.contains(&config.gateway.listening_address) {
        show_binding_warning(&config.gateway.listening_address.to_string());
    }

    let node_details = node_details(&config)?;
    let gateway = crate::node::create_gateway(config, Some(nr_opts), custom_mixnet).await?;
    eprintln!(
        "\nTo bond your gateway you will need to install the Nym wallet, go to https://nymtech.net/get-involved and select the Download button.\n\
         Select the correct version and install it to your machine. You will need to provide the following: \n ");
    output.to_stdout(&node_details);

    gateway.run().await
}
