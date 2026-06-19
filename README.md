# inherit_unlock

## Project Title
inherit_unlock — Decentralized Digital Inheritance & Dead-Man Switch on Stellar

## Project Description
Billions of dollars in digital assets are permanently lost every year because owners pass away, become incapacitated, or simply lose access to their keys before they can hand them over. `inherit_unlock` is a Soroban smart contract that lets any Stellar account owner pre-register a trusted beneficiary together with an "inactivity window"; as long as the owner periodically sends an on-chain heartbeat, nothing changes, but the moment they go silent for longer than the configured window, the beneficiary can step in and provably claim the vault, all without a centralized custodian, lawyer, or off-chain trigger.

## Project Vision
Our long-term vision is to make on-chain identity and asset succession a first-class primitive of the Stellar ecosystem. Just as people set up wills and emergency contacts in the physical world, every Stellar user should have a transparent, programmable continuity plan baked into their account. By starting from a minimal dead-man switch and expanding into multi-beneficiary allocations, social-recovery integrations, and SEP-30 recovery server interop, `inherit_unlock` aims to be the default "what happens if I'm gone" layer for wallets, DAOs, and institutional treasuries built on Stellar.

## Key Features
- **Owner-controlled vault setup** — `set_beneficiary` registers a beneficiary, an inactivity period (in seconds), and starts the dead-man timer in a single signed call.
- **Periodic proof-of-life** — `heartbeat` lets the owner reset the inactivity countdown at any time with a cheap, gas-efficient transaction.
- **Trust-minimized claiming** — `claim` allows the designated beneficiary, and only that beneficiary, to take over the vault after the inactivity window has fully elapsed; claims are one-shot and recorded on-chain.
- **Live beneficiary rotation** — `change_beneficiary` lets the owner update who inherits while they are still active, and is itself treated as a heartbeat.
- **Transparent read-only state** — `get_last_heartbeat`, `get_beneficiary`, `get_inactivity_period`, `is_claimable`, and `is_claimed` expose the entire vault status to wallets, dashboards, and monitoring bots.
- **Per-owner isolation & strict auth** — every storage entry is keyed by the owner address and every state change is gated by `require_auth`, so one deployed contract safely serves an unlimited number of independent users.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** identity dApp — see `contracts/inherit_unlock/src/lib.rs` for the full inherit_unlock business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CDFCSE5QEGVSQAQMQV53Q77CFUZALJ7JPNQ4L7FMSTDZ5GTANET22F27`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/16cb32e4551616a6f8c4afb2caef40596e9173b95e31d4c4601e4455c3116504`


## Future Scope
- **Multi-beneficiary splits**: distribute inheritance to several addresses with configurable weights (e.g., 60% to child, 30% to spouse, 10% to charity DAO).
- **Tiered inactivity windows**: warning-period beneficiaries (read-only access) followed by full-claim beneficiaries, mirroring real-world executor/legatee roles.
- **Real asset custody**: extend the contract to hold and release native XLM, custom Stellar Classic assets, and Soroban tokens via SAC, instead of only signaling "claim rights".
- **Social recovery & guardians**: combine the dead-man switch with M-of-N guardian approval to defend against accidental silence or compromised heartbeats.
- **SEP-30 recovery server integration**: plug into Stellar's standard recovery flow so wallets like Freighter and Lobstr can offer "inheritance mode" out of the box.
- **Frontend dashboard & notifications**: a Next.js dApp that visualizes every vault's countdown, sends email/Telegram reminders before expiry, and lets beneficiaries claim in one click.
- **Mainnet hardening**: formal verification of the timer logic, fuzz-tested storage migration, and a public bug-bounty program before promoting the contract from Testnet to Mainnet.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `inherit_unlock` (identity)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
