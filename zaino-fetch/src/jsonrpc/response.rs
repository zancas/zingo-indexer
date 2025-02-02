//! Request and response types for jsonRPC client.

use indexmap::IndexMap;
use serde::Deserialize;

use crate::primitives::{
    address::TransparentAddress,
    block::{BlockHash, SerializedBlock},
    chain::{ConsensusBranchIdHex, NetworkUpgradeInfo, TipConsensusBranch},
    height::ChainHeight,
    transaction::{
        BlockCommitmentTreeSize, CommitmentTreestate, NoteCommitmentSubtreeIndex, OrchardTreestate,
        SaplingTreestate, SerializedTransaction, SubtreeRpcData, TransactionHash, ZcashScript,
    },
};

/// Response to a `getinfo` RPC request.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_info`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GetInfoResponse {
    /// The node version build number
    pub build: String,
    /// The server sub-version identifier, used as the network protocol user-agent
    pub subversion: String,
}

/// Response to a `getblockchaininfo` RPC request.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_blockchain_info`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GetBlockchainInfoResponse {
    /// Current network name as defined in BIP70 (main, test, regtest)
    pub chain: String,

    /// The current number of blocks processed in the server, numeric
    pub blocks: ChainHeight,

    /// The hash of the currently best block, in big-endian order, hex-encoded
    #[serde(rename = "bestblockhash", with = "hex")]
    pub best_block_hash: BlockHash,

    /// If syncing, the estimated height of the chain, else the current best height, numeric.
    ///
    /// In Zebra, this is always the height estimate, so it might be a little inaccurate.
    #[serde(rename = "estimatedheight")]
    pub estimated_height: ChainHeight,

    /// Status of network upgrades
    pub upgrades: IndexMap<ConsensusBranchIdHex, NetworkUpgradeInfo>,

    /// Branch IDs of the current and upcoming consensus rules
    pub consensus: TipConsensusBranch,
}

/// The transparent balance of a set of addresses.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_address_balance`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GetBalanceResponse {
    /// The total transparent balance.
    pub balance: u64,
}

/// Contains the hex-encoded hash of the sent transaction.
///
/// This is used for the output parameter of [`JsonRpcConnector::send_raw_transaction`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SendTransactionResponse(#[serde(with = "hex")] pub TransactionHash);

/// Response to a `getbestblockhash` and `getblockhash` RPC request.
///
/// Contains the hex-encoded hash of the requested block.
///
/// Also see the notes for the [`Rpc::get_best_block_hash`] and `get_block_hash` methods.
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct GetBlockHash(#[serde(with = "hex")] pub BlockHash);

impl Default for GetBlockHash {
    fn default() -> Self {
        GetBlockHash(BlockHash([0; 32]))
    }
}

/// Contains the hex-encoded hash of the sent transaction.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_block`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum GetBlockResponse {
    /// The request block, hex-encoded.
    Raw(#[serde(with = "hex")] SerializedBlock),
    /// The block object.
    Object {
        /// The hash of the requested block.
        hash: GetBlockHash,

        /// The number of confirmations of this block in the best chain,
        /// or -1 if it is not in the best chain.
        confirmations: i64,

        /// The height of the requested block.
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<ChainHeight>,

        /// The height of the requested block.
        #[serde(skip_serializing_if = "Option::is_none")]
        time: Option<i64>,

        /// List of transaction IDs in block order, hex-encoded.
        tx: Vec<String>,

        /// Information about the note commitment trees.
        trees: BlockCommitmentTreeSize,
    },
}

/// Contains the hex-encoded hash of the requested block.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_best_block_hash`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct BestBlockHashResponse(#[serde(with = "hex")] pub BlockHash);

/// Vec of transaction ids, as a JSON array.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_raw_mempool`] and [`JsonRpcConnector::get_address_txids`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TxidsResponse {
    /// Vec of txids.
    pub transactions: Vec<String>,
}

impl<'de> Deserialize<'de> for TxidsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;

        let transactions = v
            .as_array()
            .ok_or_else(|| serde::de::Error::custom("Expected the JSON to be an array"))?
            .iter()
            .filter_map(|item| item.as_str().map(String::from))
            .collect::<Vec<String>>();

        Ok(TxidsResponse { transactions })
    }
}

/// Contains the hex-encoded Sapling & Orchard note commitment trees, and their
/// corresponding [`block::Hash`], [`Height`], and block time.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_treestate`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct GetTreestateResponse {
    /// The block height corresponding to the treestate, numeric.
    pub height: i32,

    /// The block hash corresponding to the treestate, hex-encoded.
    pub hash: String,

    /// Unix time when the block corresponding to the treestate was mined, numeric.
    ///
    /// UTC seconds since the Unix 1970-01-01 epoch.
    pub time: u32,

    /// A treestate containing a Sapling note commitment tree, hex-encoded.
    pub sapling: SaplingTreestate,

    /// A treestate containing an Orchard note commitment tree, hex-encoded.
    pub orchard: OrchardTreestate,
}

impl<'de> Deserialize<'de> for GetTreestateResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        let height = v["height"]
            .as_i64()
            .ok_or_else(|| serde::de::Error::missing_field("height"))? as i32;
        let hash = v["hash"]
            .as_str() // This directly accesses the string value
            .ok_or_else(|| serde::de::Error::missing_field("hash"))? // Converts Option to Result
            .to_string();
        let time = v["time"]
            .as_i64()
            .ok_or_else(|| serde::de::Error::missing_field("time"))? as u32;
        let sapling_final_state = v["sapling"]["commitments"]["finalState"]
            .as_str()
            .ok_or_else(|| serde::de::Error::missing_field("sapling final state"))?
            .to_string();
        let orchard_final_state = v["orchard"]["commitments"]["finalState"]
            .as_str()
            .ok_or_else(|| serde::de::Error::missing_field("orchard final state"))?
            .to_string();
        Ok(GetTreestateResponse {
            height,
            hash,
            time,
            sapling: SaplingTreestate {
                commitments: CommitmentTreestate {
                    final_state: sapling_final_state,
                },
            },
            orchard: OrchardTreestate {
                commitments: CommitmentTreestate {
                    final_state: orchard_final_state,
                },
            },
        })
    }
}

/// Contains raw transaction, encoded as hex bytes.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_raw_transaction`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum GetTransactionResponse {
    /// The raw transaction, encoded as hex bytes.
    Raw(#[serde(with = "hex")] SerializedTransaction),
    /// The transaction object.
    Object {
        /// The raw transaction, encoded as hex bytes.
        #[serde(with = "hex")]
        hex: SerializedTransaction,
        /// The height of the block in the best chain that contains the transaction, or -1 if
        /// the transaction is in the mempool.
        height: i32,
        /// The confirmations of the block in the best chain that contains the transaction,
        /// or 0 if the transaction is in the mempool.
        confirmations: u32,
    },
}

impl<'de> Deserialize<'de> for GetTransactionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        if v.get("height").is_some() && v.get("confirmations").is_some() {
            let hex = serde_json::from_value(v["hex"].clone()).map_err(serde::de::Error::custom)?;
            let height = v["height"]
                .as_i64()
                .ok_or_else(|| serde::de::Error::custom("Missing or invalid height"))?
                as i32;
            let confirmations = v["confirmations"]
                .as_u64()
                .ok_or_else(|| serde::de::Error::custom("Missing or invalid confirmations"))?
                as u32;
            let obj = GetTransactionResponse::Object {
                hex,
                height,
                confirmations,
            };
            Ok(obj)
        } else if v.get("hex").is_some() && v.get("txid").is_some() {
            let hex = serde_json::from_value(v["hex"].clone()).map_err(serde::de::Error::custom)?;
            let obj = GetTransactionResponse::Object {
                hex,
                height: -1,
                confirmations: 0,
            };
            Ok(obj)
        } else {
            let raw = GetTransactionResponse::Raw(
                serde_json::from_value(v.clone()).map_err(serde::de::Error::custom)?,
            );
            Ok(raw)
        }
    }
}

/// *** THE FOLLOWING CODE IS CURRENTLY UNUSED BY ZINGO-PROXY AND UNTESTED! ***
/// ***                           TEST BEFORE USE                           ***

/// Contains the Sapling or Orchard pool label, the index of the first subtree in the list,
/// and a list of subtree roots and end heights.
///
/// This is used for the output parameter of [`JsonRpcConnector::get_subtrees_by_index`].
///
/// *** UNTESTED - TEST BEFORE USE ***
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GetSubtreesResponse {
    /// The shielded pool to which the subtrees belong.
    pub pool: String,

    /// The index of the first subtree.
    pub start_index: NoteCommitmentSubtreeIndex,

    /// A sequential list of complete subtrees, in `index` order.
    ///
    /// The generic subtree root type is a hex-encoded Sapling or Orchard subtree root string.
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subtrees: Vec<SubtreeRpcData>,
}

/// This is used for the output parameter of [`JsonRpcConnector::get_address_utxos`].
///
/// *** UNTESTED - TEST BEFORE USE ***
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GetUtxosResponse {
    /// The transparent address, base58check encoded
    pub address: TransparentAddress,

    /// The output txid, in big-endian order, hex-encoded
    #[serde(with = "hex")]
    pub txid: TransactionHash,

    /// The transparent output index, numeric
    #[serde(rename = "outputIndex")]
    pub output_index: u32,

    /// The transparent output script, hex encoded
    #[serde(with = "hex")]
    pub script: ZcashScript,

    /// The amount of zatoshis in the transparent output
    pub satoshis: u64,

    /// The block height, numeric.
    pub height: ChainHeight,
}
