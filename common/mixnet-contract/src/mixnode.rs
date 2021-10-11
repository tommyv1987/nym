// due to code generated by JsonSchema
#![allow(clippy::field_reassign_with_default)]

use crate::error::MixnetContractError;
use crate::{IdentityKey, SphinxKey};
use config::defaults::{ALPHA, DEFAULT_OPERATOR_EPOCH_COST};
use cosmwasm_std::{coin, Addr, Coin, Uint128};
use num::rational::Ratio;
use num::ToPrimitive;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::Ordering;
use std::fmt::Display;

use crate::current_block_height;

const DEFAULT_PROFIT_MARGIN: f64 = 0.1;

#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize, JsonSchema)]
pub struct MixNode {
    pub host: String,
    pub mix_port: u16,
    pub verloc_port: u16,
    pub http_api_port: u16,
    pub sphinx_key: SphinxKey,
    /// Base58 encoded ed25519 EdDSA public key.
    pub identity_key: IdentityKey,
    pub version: String,
}

#[derive(
    Copy, Clone, Debug, Serialize_repr, PartialEq, PartialOrd, Deserialize_repr, JsonSchema,
)]
#[repr(u8)]
pub enum Layer {
    Gateway = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, JsonSchema, PartialEq, Serialize, Deserialize, Copy)]
pub struct NodeRewardParams {
    income_global_mix: f64,
    k: f64,
    one_over_k: f64,
    performance: f64,
    reward_blockstamp: Option<u64>,
    total_mix_stake: Uint128,
    uptime: f64,
}

impl NodeRewardParams {
    pub fn new(
        income_global_mix: f64,
        k: f64,
        performance: f64,
        reward_blockstamp: Option<u64>,
        total_mix_stake: u128,
        uptime: f64,
    ) -> NodeRewardParams {
        NodeRewardParams {
            income_global_mix,
            k,
            one_over_k: 1. / k,
            performance,
            reward_blockstamp,
            total_mix_stake: Uint128(total_mix_stake),
            uptime,
        }
    }

    pub fn operator_cost(&self) -> f64 {
        self.uptime / 100. * DEFAULT_OPERATOR_EPOCH_COST as f64
    }

    pub fn set_reward_blockstamp(&mut self, blockstamp: u64) {
        self.reward_blockstamp = Some(blockstamp);
    }

    pub fn alpha(&self) -> f64 {
        ALPHA
    }

    pub fn income_global_mix(&self) -> f64 {
        self.income_global_mix
    }

    pub fn k(&self) -> f64 {
        self.k
    }

    pub fn performance(&self) -> f64 {
        self.performance
    }

    pub fn total_mix_stake(&self) -> Uint128 {
        self.total_mix_stake
    }

    pub fn reward_blockstamp(&self) -> Result<u64, MixnetContractError> {
        self.reward_blockstamp
            .ok_or(MixnetContractError::BlockstampNotSet)
    }

    pub fn one_over_k(&self) -> f64 {
        self.one_over_k
    }
}

#[derive(Debug)]
pub struct NodeRewardResult {
    reward: f64,
    lambda: f64,
    sigma: f64,
}

impl NodeRewardResult {
    pub fn reward(&self) -> f64 {
        self.reward
    }

    pub fn lambda(&self) -> f64 {
        self.lambda
    }

    pub fn sigma(&self) -> f64 {
        self.sigma
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixNodeBond {
    pub bond_amount: Coin,
    pub total_delegation: Coin,
    pub owner: Addr,
    pub layer: Layer,
    #[serde(default = "current_block_height")]
    pub block_height: u64,
    pub mix_node: MixNode,
    pub profit_margin: Option<f64>,
}

impl MixNodeBond {
    pub fn new(
        bond_amount: Coin,
        owner: Addr,
        layer: Layer,
        block_height: u64,
        mix_node: MixNode,
        profit_margin: Option<f64>,
    ) -> Self {
        MixNodeBond {
            total_delegation: coin(0, &bond_amount.denom),
            bond_amount,
            owner,
            layer,
            block_height,
            mix_node,
            profit_margin,
        }
    }

    pub fn profit_margin(&self) -> f64 {
        if let Some(margin) = self.profit_margin {
            margin
        } else {
            DEFAULT_PROFIT_MARGIN
        }
    }

    pub fn identity(&self) -> &String {
        &self.mix_node.identity_key
    }

    pub fn bond_amount(&self) -> Coin {
        self.bond_amount.clone()
    }

    pub fn owner(&self) -> &Addr {
        &self.owner
    }

    pub fn mix_node(&self) -> &MixNode {
        &self.mix_node
    }

    pub fn total_delegation(&self) -> Coin {
        self.total_delegation.clone()
    }

    pub fn bond_to_total_stake_ratio(&self, total_stake: Uint128) -> Ratio<u128> {
        Ratio::new(self.bond_amount().amount.u128(), total_stake.u128())
    }

    pub fn bond_to_total_stake_f64(
        &self,
        total_stake: Uint128,
    ) -> Result<f64, MixnetContractError> {
        let ratio = self.bond_to_total_stake_ratio(total_stake);
        if let Some(f) = ratio.to_f64() {
            Ok(f)
        } else {
            Err(MixnetContractError::InvalidRatio(
                *ratio.numer(),
                *ratio.denom(),
            ))
        }
    }

    pub fn stake_to_total_stake_ratio(&self, total_stake: Uint128) -> Ratio<u128> {
        Ratio::new(
            self.bond_amount().amount.u128() + self.total_delegation().amount.u128(),
            total_stake.u128(),
        )
    }

    pub fn stake_to_total_stake_f64(
        &self,
        total_stake: Uint128,
    ) -> Result<f64, MixnetContractError> {
        let ratio = self.stake_to_total_stake_ratio(total_stake);
        if let Some(f) = ratio.to_f64() {
            Ok(f)
        } else {
            Err(MixnetContractError::InvalidRatio(
                *ratio.numer(),
                *ratio.denom(),
            ))
        }
    }

    pub fn lambda(&self, params: &NodeRewardParams) -> Result<f64, MixnetContractError> {
        let bond_to_total_stake_ratio = self.bond_to_total_stake_f64(params.total_mix_stake)?;
        Ok(bond_to_total_stake_ratio.min(params.one_over_k))
    }

    pub fn sigma(&self, params: &NodeRewardParams) -> Result<f64, MixnetContractError> {
        let stake_to_total_stake_ratio = self.stake_to_total_stake_f64(params.total_mix_stake)?;
        Ok(stake_to_total_stake_ratio.min(params.one_over_k))
    }

    pub fn reward(
        &self,
        params: &NodeRewardParams,
    ) -> Result<NodeRewardResult, MixnetContractError> {
        // Assuming uniform work distribution across the network this is one_over_k * k
        let omega_k = 1.;
        let lambda = self.lambda(params)?;
        let sigma = self.sigma(params)?;
        let reward = params.performance
            * params.income_global_mix
            * (sigma * omega_k + params.alpha() * lambda * (sigma * params.k))
            / (1. + params.alpha());

        Ok(NodeRewardResult {
            reward,
            lambda,
            sigma,
        })
    }

    pub fn node_profit(&self, params: &NodeRewardParams) -> Result<f64, MixnetContractError> {
        Ok(self.reward(params)?.reward() - params.operator_cost())
    }

    pub fn operator_profit(&self, params: &NodeRewardParams) -> Result<f64, MixnetContractError> {
        let reward_result = self.reward(params)?;
        let profit = reward_result.reward() - params.operator_cost();
        Ok(((self.profit_margin()
            + (1. - self.profit_margin()) * (reward_result.lambda() / reward_result.sigma()))
            * profit)
            .max(0.))
    }

    pub fn sigma_ratio(&self, params: &NodeRewardParams) -> Result<f64, MixnetContractError> {
        if self.stake_to_total_stake_f64(params.total_mix_stake)? < params.one_over_k {
            self.stake_to_total_stake_f64(params.total_mix_stake)
        } else {
            Ok(params.one_over_k)
        }
    }

    pub fn reward_delegation(
        &self,
        delegation_amount: Uint128,
        params: &NodeRewardParams,
    ) -> Result<Uint128, MixnetContractError> {
        let scaled_delegation_amount =
            Ratio::new(delegation_amount.u128(), params.total_mix_stake.u128())
                .to_f64()
                .ok_or_else(|| {
                    MixnetContractError::InvalidRatio(
                        delegation_amount.u128(),
                        params.total_mix_stake.u128(),
                    )
                })?;

        let delegation_over_sigma_f64 = scaled_delegation_amount / self.sigma(params)?;
        // If we can't resolve the ration we move on, and the delegation does not get rewarded
        Ok(Uint128(
            ((1. - self.profit_margin()) * delegation_over_sigma_f64 * self.node_profit(params)?)
                .max(0.) as u128,
        ))
    }
}

impl PartialOrd for MixNodeBond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // first remove invalid cases
        if self.bond_amount.denom != self.total_delegation.denom {
            return None;
        }

        if other.bond_amount.denom != other.total_delegation.denom {
            return None;
        }

        if self.bond_amount.denom != other.bond_amount.denom {
            return None;
        }

        // try to order by total bond + delegation
        let total_cmp = (self.bond_amount.amount + self.total_delegation.amount)
            .partial_cmp(&(self.bond_amount.amount + self.total_delegation.amount))?;

        if total_cmp != Ordering::Equal {
            return Some(total_cmp);
        }

        // then if those are equal, prefer higher bond over delegation
        let bond_cmp = self
            .bond_amount
            .amount
            .partial_cmp(&other.bond_amount.amount)?;
        if bond_cmp != Ordering::Equal {
            return Some(bond_cmp);
        }

        // then look at delegation (I'm not sure we can get here, but better safe than sorry)
        let delegation_cmp = self
            .total_delegation
            .amount
            .partial_cmp(&other.total_delegation.amount)?;
        if delegation_cmp != Ordering::Equal {
            return Some(delegation_cmp);
        }

        // then check block height
        let height_cmp = self.block_height.partial_cmp(&other.block_height)?;
        if height_cmp != Ordering::Equal {
            return Some(height_cmp);
        }

        // finally go by the rest of the fields in order. It doesn't really matter at this point
        // but we should be deterministic.
        let owner_cmp = self.owner.partial_cmp(&other.owner)?;
        if owner_cmp != Ordering::Equal {
            return Some(owner_cmp);
        }

        let layer_cmp = self.layer.partial_cmp(&other.layer)?;
        if layer_cmp != Ordering::Equal {
            return Some(layer_cmp);
        }

        self.mix_node.partial_cmp(&other.mix_node)
    }
}

impl Display for MixNodeBond {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "amount: {} {}, owner: {}, identity: {}",
            self.bond_amount.amount, self.bond_amount.denom, self.owner, self.mix_node.identity_key
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct PagedMixnodeResponse {
    pub nodes: Vec<MixNodeBond>,
    pub per_page: usize,
    pub start_next_after: Option<IdentityKey>,
}

impl PagedMixnodeResponse {
    pub fn new(
        nodes: Vec<MixNodeBond>,
        per_page: usize,
        start_next_after: Option<IdentityKey>,
    ) -> Self {
        PagedMixnodeResponse {
            nodes,
            per_page,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixOwnershipResponse {
    pub address: Addr,
    pub has_node: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixnode_fixture() -> MixNode {
        MixNode {
            host: "1.1.1.1".to_string(),
            mix_port: 123,
            verloc_port: 456,
            http_api_port: 789,
            sphinx_key: "sphinxkey".to_string(),
            identity_key: "identitykey".to_string(),
            version: "0.11.0".to_string(),
        }
    }

    #[test]
    fn mixnode_bond_partial_ord() {
        let _150foos = Coin::new(150, "foo");
        let _50foos = Coin::new(50, "foo");
        let _0foos = Coin::new(0, "foo");

        let mix1 = MixNodeBond {
            bond_amount: _150foos.clone(),
            total_delegation: _50foos.clone(),
            owner: Addr::unchecked("foo1"),
            layer: Layer::One,
            block_height: 100,
            mix_node: mixnode_fixture(),
            profit_margin: None,
        };

        let mix2 = MixNodeBond {
            bond_amount: _150foos.clone(),
            total_delegation: _50foos.clone(),
            owner: Addr::unchecked("foo2"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            profit_margin: None,
        };

        let mix3 = MixNodeBond {
            bond_amount: _50foos,
            total_delegation: _150foos.clone(),
            owner: Addr::unchecked("foo3"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            profit_margin: None,
        };

        let mix4 = MixNodeBond {
            bond_amount: _150foos.clone(),
            total_delegation: _0foos.clone(),
            owner: Addr::unchecked("foo4"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            profit_margin: None,
        };

        let mix5 = MixNodeBond {
            bond_amount: _0foos,
            total_delegation: _150foos,
            owner: Addr::unchecked("foo5"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            profit_margin: None,
        };

        // summary:
        // mix1: 150bond + 50delegation, foo1, 100
        // mix2: 150bond + 50delegation, foo2, 120
        // mix3: 50bond + 150delegation, foo3, 120
        // mix4: 150bond + 0delegation, foo4, 120
        // mix5: 0bond + 150delegation, foo5, 120

        // highest total bond+delegation is used
        // then bond followed by delegation
        // finally just the rest of the fields

        // mix1 has higher total than mix4 or mix5
        assert!(mix1 > mix4);
        assert!(mix1 > mix5);

        // mix1 has the same total as mix3, however, mix1 has more tokens in bond
        assert!(mix1 > mix3);
        // same case for mix4 and mix5
        assert!(mix4 > mix5);

        // same bond and delegation, so it's just ordered by height
        assert!(mix1 < mix2);
    }
}
