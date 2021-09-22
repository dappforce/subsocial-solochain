use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use subsocial_primitives::Balance;
use subsocial_runtime::{
    AccountId, GenesisConfig, SessionKeys, StakerStatus,
    constants::currency::DOLLARS,
};
use subsocial_runtime::{
    SessionConfig, StakingConfig, BalancesConfig, SudoConfig,
    ElectionsConfig, TechnicalCommitteeConfig,
    SpacesConfig, SystemConfig, WASM_BINARY, Signature,
};
use sp_consensus_babe::AuthorityId as BabeId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    Perbill,
    traits::{Verify, IdentifyAccount},
};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn testnet_endowed_accounts() -> Vec<AccountId> {
    vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
    ]
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            // Pre-funded accounts
            testnet_endowed_accounts().iter().cloned().map(|k| (k, 100_000)).collect(),
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some(DEFAULT_PROTOCOL_ID),
        // Properties
        Some(subsocial_properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            // Pre-funded accounts
            testnet_endowed_accounts().iter().cloned().map(|k| (k, 100_000)).collect(),
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some(DEFAULT_PROTOCOL_ID),
        // Properties
        Some(subsocial_properties()),
        // Extensions
        None,
    ))
}

pub fn subsocial_staging_testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/staging.json")[..])
}

pub fn subsocial_staging_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Staging wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Subsocial Staging",
        "subsocial",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            vec![
                (
                    //5HHJDXTAre66UuHqx9vHU1L55BL1eYCdcSrW6UqCs3wLpALJ (Stash)
                    hex!["e6c7c6e02890bd7d762dadc7bf2b2bfd28931ae51b48780399f78950a477c760"].into(),
                    //5FxaTpL41CUYcgGDN2KtyVVxKxsTnZuk6Rdv72mBq9JCHKzC (Controller)
                    hex!["ac44922ae8bc22a330e3f24930eac35d3f1dba435b28c79564ba1569a4094d4b"].into(),
                    //5EdyW51GA5qcseQ86gW5j8uFf2TMcaz2z5qf27WSUk3dx1YS (GrandpaId)
                    hex!["71d83b01f2ffe5a0b44b1056c3bb6e3c537f6d9588a0342d3de6fae4b2c16442"].unchecked_into(),
                    //5HHJDXTAre66UuHqx9vHU1L55BL1eYCdcSrW6UqCs3wLpALJ (BabeId)
                    hex!["e6c7c6e02890bd7d762dadc7bf2b2bfd28931ae51b48780399f78950a477c760"].unchecked_into(),
                    //5EtLFxUfgKFYXSttrRtgSucLUuzhrvgixiaK6sjvYwyKMp3y (ImOnlineId)
                    hex!["7ccb8a1469f64cdd8aeed9f48376dc60b021bfc65d3a2f8d3a12c8b6ee77ab45"].unchecked_into(),
                    //5DXcYMa34ZJfACVqETL9JCa9Epe6TS448aZPc179BfUtUpzd (AuthorityDiscoveryId)
                    hex!["40c1f708806e670825eaeb725509e9a594d58d430f58d691d0591def9b40316e"].unchecked_into(),
                ),
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
        ),
        vec![],
        Some(TelemetryEndpoints::new(
            vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
        ).expect("Staging telemetry url is valid; qed")),
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        None,
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, u128)>,
) -> GenesisConfig {
    const STASH: Balance = 10 * DOLLARS;
    let num_endowed_accounts = endowed_accounts.len();

    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts.iter().cloned().map(|(k, b)|(k, b * DOLLARS))
                .chain(
                    initial_authorities.iter().map(|x| (x.0.clone(), STASH))
                )
                .collect(),
        }),
        pallet_indices: Default::default(),
        pallet_babe: Default::default(),
        pallet_grandpa: Default::default(),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key.clone(),
        }),
        pallet_spaces: Some(SpacesConfig {
            endowed_account: root_key,
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.0.clone(), session_keys(
                    x.2.clone(),
                    x.3.clone(),
                    x.4.clone(),
                    x.5.clone(),
                ))
            }).collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities.iter().map(|x| {
                (
                    x.0.clone(),
                    x.1.clone(),
                    STASH,
                    StakerStatus::Validator
                )
            }).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|(member, _)| (member, STASH))
                .collect(),
        }),
        pallet_im_online: Default::default(),
        pallet_authority_discovery: Default::default(),
        pallet_democracy: Default::default(),
        pallet_collective_Instance1: Default::default(),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|(member, _)| member)
                .collect(),
            phantom: Default::default(),
        }),
        pallet_membership_Instance1: Default::default(),
        pallet_treasury: Default::default(),
        pallet_vesting: Default::default(),
    }
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
