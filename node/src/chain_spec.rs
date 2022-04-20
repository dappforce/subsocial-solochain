use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use subsocial_runtime::{
	AccountId, AuraConfig, BalancesConfig,
	GenesisConfig, GrandpaConfig, UtilsConfig,
	SudoConfig, SpacesConfig, SystemConfig,
	WASM_BINARY, Signature,
};
use subsocial_primitives::Block;
use subsocial_primitives::currency::DOLLARS;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;
use serde::{Serialize, Deserialize};
use sc_chain_spec::ChainSpecExtension;

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
    /// The light sync state extension used by the sync-state rpc.
    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
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
                    authority_keys_from_seed("Alice"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.iter().cloned().map(|k| (k, 100_000)).collect(),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                true,
            )
        },
        vec![],
        None,
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Default::default(),
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
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
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.iter().cloned().map(|k| (k, 100_000)).collect(),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                true,
            )
        },
        vec![],
        None,
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Default::default(),
    ))
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial.json")[..])
}

pub fn soonsocial_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/soonsocial.json")[..])
}

pub fn subsocial_staging_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Staging wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "SoonSocial",
        "subsocial",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            vec![
                (
                    /* AuraId SR25519 */
                    hex!["acdcc09f7dc4e55353d89ed441f7a6e748a57b71fd21b641bec65e19fe1dac46"].unchecked_into(),
                    /* GrandpaId ED25519 */
                    hex!["51005ea936bf7e7118c448b20320a658ca614426fc91018182fe011f90b03141"].unchecked_into()
                ),
                (
                    /* AuraId SR25519 */
                    hex!["46c8a0343fe04761653c088c2c7bcfdcdd46a113c5cfd8cb36c8c134da274a37"].unchecked_into(),
                    /* GrandpaId ED25519 */
                    hex!["acf25bba3d550f197d2cd3012842925204d0a99f97645d7a3991c11fa50027bb"].unchecked_into()
                ),
            ],
            /* Sudo Account */
            hex!["ce7035e9f36c57ac8c3cc016b150ee5d36da10c4417c45e30c62c2f627f19d36"].into(),
            vec![
                (
                    /* Sudo Account */
                    hex!["ce7035e9f36c57ac8c3cc016b150ee5d36da10c4417c45e30c62c2f627f19d36"].into(),
                    /* Balance */
                    1_000_000_000
                ),
            ],
            // Treasury
            hex!["222c69e0bba9f913c4941aa35d3ad80164a1ade36b517383f438a52d868a880a"].into(),
            true,
        ),
        vec![],
        Some(TelemetryEndpoints::new(
            vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
        ).expect("Staging telemetry url is valid; qed")),
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        Default::default(),
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	treasury_account_id: AccountId,
	_enable_println: bool
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
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        },
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
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
