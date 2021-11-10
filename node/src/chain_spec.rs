use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use subsocial_runtime::{
	AccountId, AuraConfig, BalancesConfig,
	GenesisConfig, UtilsConfig,
	SudoConfig, SpacesConfig, SystemConfig,
	WASM_BINARY, Signature, constants::currency::DOLLARS,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;
use cumulus_primitives_core::ParaId;
use serde::{Deserialize, Serialize};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || {
            let endowed_accounts = vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
            ];

            testnet_genesis(
                wasm_binary,
                vec![
                    get_from_seed::<AuraId>("Alice"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.iter().cloned().map(|k| (k, 100_000)).collect(),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                id,
            )
        },
        vec![],
        None,
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Extensions {
            relay_chain: "kusama-local".into(), // You MUST set this to the correct network!
            para_id: id.into(),
        },
    ))
}

pub fn local_testnet_config(id: ParaId) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || {
            let endowed_accounts = vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
            ];

            testnet_genesis(
                wasm_binary,
                vec![
                    get_from_seed::<AuraId>("Alice"),
                    get_from_seed::<AuraId>("Bob"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.iter().cloned().map(|k| (k, 100_000)).collect(),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                id,
            )
        },
        vec![],
        None,
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Extensions {
            relay_chain: "kusama-local".into(), // You MUST set this to the correct network!
            para_id: id.into(),
        },
    ))
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial.json")[..])
}

pub fn subsocial_staging_config(id: ParaId) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Staging wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Subsocial Staging",
        "subsocial",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            vec![
                /* AuraId SR25519 */
                hex!["e6c7c6e02890bd7d762dadc7bf2b2bfd28931ae51b48780399f78950a477c760"].unchecked_into(),
            ],
            /* Sudo Account */
            hex!["ce7035e9f36c57ac8c3cc016b150ee5d36da10c4417c45e30c62c2f627f19d36"].into(),
            vec![
                (
                    /* Sudo Account */
                    hex!["ce7035e9f36c57ac8c3cc016b150ee5d36da10c4417c45e30c62c2f627f19d36"].into(),
                    /* Balance */
                    1_000
                ),
            ],
            // Treasury
            hex!["24d6d683750c4c10e90dd81430efec95133e1ec1f5be781d3267390d03174706"].into(),
            id,
        ),
        vec![],
        Some(TelemetryEndpoints::new(
            vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
        ).expect("Staging telemetry url is valid; qed")),
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Extensions {
            relay_chain: "kusama-local".into(), // You MUST set this to the correct network!
            para_id: id.into(),
        },
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
	initial_authorities: Vec<AuraId>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	treasury_account_id: AccountId,
    id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
        system: SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|(k, b)| (k, b * DOLLARS)).collect(),
        },
		aura: AuraConfig {
            authorities: initial_authorities,
        },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        parachain_info: subsocial_runtime::ParachainInfoConfig { parachain_id: id },
		sudo: SudoConfig {
            key: root_key.clone(),
        },
		utils: UtilsConfig {
            treasury_account: treasury_account_id,
        },
		spaces: SpacesConfig {
            endowed_account: root_key,
        },
	}
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 11.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
