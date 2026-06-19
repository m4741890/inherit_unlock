#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

/// Storage keys used by the InheritUnlock contract. Each entry is keyed by the
/// owner address so a single deployed contract can host many independent
/// inheritance vaults.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Address that will inherit access if the owner goes silent.
    Beneficiary(Address),
    /// Inactivity window, in seconds, that must elapse before a claim is valid.
    Inactivity(Address),
    /// Unix timestamp of the owner's most recent proof-of-life.
    Heartbeat(Address),
    /// Flag set to `true` once the beneficiary has successfully claimed.
    Claimed(Address),
}

#[contract]
pub struct InheritUnlock;

#[contractimpl]
impl InheritUnlock {
    /// Register (or fully reset) an inheritance vault.
    ///
    /// The `owner` designates a `beneficiary` and the `inactivity_period`
    /// (in seconds) after which the beneficiary is allowed to claim the
    /// vault if the owner stops sending heartbeats. The owner's signature
    /// is required and the dead-man timer starts at the current ledger
    /// timestamp.
    pub fn set_beneficiary(
        env: Env,
        owner: Address,
        beneficiary: Address,
        inactivity_period: u64,
    ) {
        owner.require_auth();

        if inactivity_period == 0 {
            panic!("inactivity_period must be greater than zero");
        }
        if owner == beneficiary {
            panic!("beneficiary cannot be the owner");
        }

        // A vault that was already claimed by the beneficiary is final and
        // cannot be reconfigured by the (presumed-missing) owner.
        let claimed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(owner.clone()))
            .unwrap_or(false);
        if claimed {
            panic!("vault already claimed by beneficiary");
        }

        let now = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Beneficiary(owner.clone()), &beneficiary);
        env.storage()
            .persistent()
            .set(&DataKey::Inactivity(owner.clone()), &inactivity_period);
        env.storage()
            .persistent()
            .set(&DataKey::Heartbeat(owner.clone()), &now);

        env.events().publish(
            (symbol_short!("set_ben"), owner),
            (beneficiary, inactivity_period, now),
        );
    }

    /// Record a proof-of-life from the owner.
    ///
    /// Resets the dead-man timer back to the current ledger timestamp, so the
    /// inactivity countdown starts over. The owner's signature is required.
    pub fn heartbeat(env: Env, owner: Address) {
        owner.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Beneficiary(owner.clone()))
        {
            panic!("no vault registered for this owner");
        }

        let claimed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(owner.clone()))
            .unwrap_or(false);
        if claimed {
            panic!("vault already claimed by beneficiary");
        }

        let now = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Heartbeat(owner.clone()), &now);

        env.events()
            .publish((symbol_short!("hbeat"), owner), now);
    }

    /// Claim a dormant vault on behalf of the designated beneficiary.
    ///
    /// The call must be signed by `beneficiary` and is only accepted once the
    /// configured inactivity window has fully elapsed since the owner's last
    /// heartbeat. Marks the vault as claimed so it cannot be claimed again
    /// or hijacked by a later reconfiguration.
    pub fn claim(env: Env, beneficiary: Address, owner: Address) {
        beneficiary.require_auth();

        let registered: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Beneficiary(owner.clone()))
            .unwrap_or_else(|| panic!("no vault registered for this owner"));

        if registered != beneficiary {
            panic!("caller is not the designated beneficiary");
        }

        let already: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(owner.clone()))
            .unwrap_or(false);
        if already {
            panic!("vault already claimed");
        }

        let last: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Heartbeat(owner.clone()))
            .unwrap_or_else(|| panic!("missing heartbeat"));
        let window: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Inactivity(owner.clone()))
            .unwrap_or_else(|| panic!("missing inactivity period"));

        let now = env.ledger().timestamp();
        if now.saturating_sub(last) < window {
            panic!("owner is still active, claim window not open");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Claimed(owner.clone()), &true);

        env.events()
            .publish((symbol_short!("claim"), owner), (beneficiary, now));
    }

    /// Replace the designated beneficiary while the owner is still active.
    ///
    /// The owner's signature is required, the vault must not have been
    /// claimed yet, and the action is treated as a heartbeat (the timer is
    /// refreshed) because performing it is itself proof of life.
    pub fn change_beneficiary(env: Env, owner: Address, new_beneficiary: Address) {
        owner.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Beneficiary(owner.clone()))
        {
            panic!("no vault registered for this owner");
        }

        let claimed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(owner.clone()))
            .unwrap_or(false);
        if claimed {
            panic!("vault already claimed by beneficiary");
        }

        if owner == new_beneficiary {
            panic!("beneficiary cannot be the owner");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Beneficiary(owner.clone()), &new_beneficiary);

        let now = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Heartbeat(owner.clone()), &now);

        env.events().publish(
            (symbol_short!("chg_ben"), owner),
            (new_beneficiary, now),
        );
    }

    /// Read-only: timestamp (seconds) of the owner's last heartbeat.
    pub fn get_last_heartbeat(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Heartbeat(owner))
            .unwrap_or_else(|| panic!("no vault registered for this owner"))
    }

    /// Read-only: returns `true` when the inactivity window has fully elapsed
    /// since the last heartbeat AND the vault has not already been claimed.
    pub fn is_claimable(env: Env, owner: Address) -> bool {
        let claimed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(owner.clone()))
            .unwrap_or(false);
        if claimed {
            return false;
        }

        let last: u64 = match env
            .storage()
            .persistent()
            .get(&DataKey::Heartbeat(owner.clone()))
        {
            Some(t) => t,
            None => return false,
        };
        let window: u64 = match env
            .storage()
            .persistent()
            .get(&DataKey::Inactivity(owner))
        {
            Some(w) => w,
            None => return false,
        };

        env.ledger().timestamp().saturating_sub(last) >= window
    }

    /// Read-only: address currently designated as the beneficiary.
    pub fn get_beneficiary(env: Env, owner: Address) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Beneficiary(owner))
            .unwrap_or_else(|| panic!("no vault registered for this owner"))
    }

    /// Read-only: configured inactivity window in seconds.
    pub fn get_inactivity_period(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Inactivity(owner))
            .unwrap_or_else(|| panic!("no vault registered for this owner"))
    }

    /// Read-only: `true` once the beneficiary has successfully claimed.
    pub fn is_claimed(env: Env, owner: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(owner))
            .unwrap_or(false)
    }
}
