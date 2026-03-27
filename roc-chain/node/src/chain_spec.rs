use roc_runtime::{AccountId, RuntimeGenesisConfig, Signature, WASM_BINARY};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from a seed string.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from a seed string.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate authority keys (Aura + Grandpa) from a seed string.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development WASM not available".to_string())?,
        None,
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(testnet_genesis(
        vec![authority_keys_from_seed("Alice")],
        vec![],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
    ))
    .build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development WASM not available".to_string())?,
        None,
    )
    .with_name("Local Testnet")
    .with_id("local_testnet")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        vec![],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
    ))
    .build())
}

/// BlocRoc 4-validator testnet — named after iconic music venues.
///
/// Validators:
/// - The Roxy        (alice keys)
/// - Red Rocks       (bob keys)
/// - House of Blues   (charlie keys)
/// - Local Dive Bar   (dave keys)
///
/// All six well-known dev accounts are endowed so integration tests
/// can submit extrinsics without funding first.
pub fn blocroc_testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development WASM not available".to_string())?,
        None,
    )
    .with_name("BlocRoc Testnet")
    .with_id("blocroc_testnet")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),   // The Roxy
            authority_keys_from_seed("Bob"),      // Red Rocks
            authority_keys_from_seed("Charlie"),  // House of Blues
            authority_keys_from_seed("Dave"),     // Local Dive Bar
        ],
        vec![],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
    ))
    .with_protocol_id("blocroc")
    .build())
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _initial_nominators: Vec<AccountId>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
) -> serde_json::Value {
    let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        ]
    });

    const ENDOWMENT: u128 = 1_000_000_000_000_000_000; // 1M ROC tokens (18 decimals)

    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().map(|a| (a, ENDOWMENT)).collect::<Vec<_>>(),
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1u64)).collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key),
        },
    })
}
