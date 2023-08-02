// Copyright 2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::nyxd::contract_traits::NymContractsProvider;
use crate::nyxd::cosmwasm_client::types::ExecuteResult;
use crate::nyxd::error::NyxdError;
use crate::nyxd::{Coin, Fee, SigningCosmWasmClient};
use crate::signing::signer::OfflineSigner;
use async_trait::async_trait;
use cosmwasm_std::{to_binary, CosmosMsg, WasmMsg};
use cw3::Vote;
use nym_coconut_bandwidth_contract_common::msg::ExecuteMsg as CoconutBandwidthExecuteMsg;
use nym_multisig_contract_common::msg::ExecuteMsg as MultisigExecuteMsg;

#[async_trait]
pub trait MultisigSigningClient: NymContractsProvider {
    async fn execute_multisig_contract(
        &self,
        fee: Option<Fee>,
        msg: MultisigExecuteMsg,
        memo: String,
        funds: Vec<Coin>,
    ) -> Result<ExecuteResult, NyxdError>;

    async fn propose_release_funds(
        &self,
        title: String,
        blinded_serial_number: String,
        voucher_value: Coin,
        fee: Option<Fee>,
    ) -> Result<ExecuteResult, NyxdError> {
        let coconut_bandwidth_contract_address = self
            .coconut_bandwidth_contract_address()
            .ok_or_else(|| NyxdError::unavailable_contract_address("coconut bandwidth contract"))?;

        let release_funds_req = CoconutBandwidthExecuteMsg::ReleaseFunds {
            funds: voucher_value.into(),
        };
        let release_funds_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: coconut_bandwidth_contract_address.to_string(),
            msg: to_binary(&release_funds_req)?,
            funds: vec![],
        });
        let req = MultisigExecuteMsg::Propose {
            title,
            description: blinded_serial_number,
            msgs: vec![release_funds_msg],
            latest: None,
        };
        self.execute_multisig_contract(
            fee,
            req,
            "Multisig::Propose::Execute::ReleaseFunds".to_string(),
            vec![],
        )
        .await
    }

    async fn vote_proposal(
        &self,
        proposal_id: u64,
        vote_yes: bool,
        fee: Option<Fee>,
    ) -> Result<ExecuteResult, NyxdError> {
        let vote = if vote_yes { Vote::Yes } else { Vote::No };
        let req = MultisigExecuteMsg::Vote { proposal_id, vote };
        self.execute_multisig_contract(fee, req, "Multisig::Vote".to_string(), vec![])
            .await
    }

    async fn execute_proposal(
        &self,
        proposal_id: u64,
        fee: Option<Fee>,
    ) -> Result<ExecuteResult, NyxdError> {
        let req = MultisigExecuteMsg::Execute { proposal_id };
        self.execute_multisig_contract(fee, req, "Multisig::Execute".to_string(), vec![])
            .await
    }
}

#[async_trait]
impl<C> MultisigSigningClient for C
where
    C: SigningCosmWasmClient + NymContractsProvider + Sync,
    NyxdError: From<<Self as OfflineSigner>::Error>,
{
    async fn execute_multisig_contract(
        &self,
        fee: Option<Fee>,
        msg: MultisigExecuteMsg,
        memo: String,
        funds: Vec<Coin>,
    ) -> Result<ExecuteResult, NyxdError> {
        let multisig_contract_address = self
            .multisig_contract_address()
            .ok_or_else(|| NyxdError::unavailable_contract_address("multisig contract"))?;

        let fee = fee.unwrap_or(Fee::Auto(Some(self.simulated_gas_multiplier())));

        let signer_address = &self.signer_addresses()?[0];
        self.execute(
            signer_address,
            multisig_contract_address,
            &msg,
            fee,
            memo,
            funds,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // it's enough that this compiles and clippy is happy about it
    async fn all_execute_variants_are_covered<C: MultisigSigningClient + Send + Sync>(
        client: C,
        msg: MultisigExecuteMsg,
    ) {
        unimplemented!()
    }
}
