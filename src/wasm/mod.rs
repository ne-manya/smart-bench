pub mod runner;
mod xts;

use crate::{Cli, Contract};
use futures::{future, TryStreamExt};
use sp_core::sr25519;
use sp_keyring::AccountKeyring;
use subxt::{DefaultConfig, PairSigner};
use xts::ContractsApi;

pub type Balance = u128;
pub type Gas = u64;
pub type AccountId = <DefaultConfig as subxt::Config>::AccountId;
pub type Hash = <DefaultConfig as subxt::Config>::Hash;
pub type Signer = PairSigner<DefaultConfig, sr25519::Pair>;

/// Trait implemented by [`smart_bench_macro::contract`] for all contract constructors.
pub trait InkConstructor: codec::Encode {
    const SELECTOR: [u8; 4];
}

/// Trait implemented by [`smart_bench_macro::contract`] for all contract messages.
pub trait InkMessage: codec::Encode {
    const SELECTOR: [u8; 4];
}

smart_bench_macro::contract!("./contracts/computation.contract");
smart_bench_macro::contract!("./contracts/pendulum_amm.contract");

pub async fn exec(cli: Cli) -> color_eyre::Result<()> {
    let alice = PairSigner::new(AccountKeyring::Alice.pair());
    let bob = AccountKeyring::Bob.to_account_id();

    let alice_acc: &sp_core::crypto::AccountId32 = alice.account_id();
    let alice_acc = alice_acc.clone().to_string();

    let mut runner = runner::BenchRunner::new(alice, &cli.url).await?;

    for contract in &cli.contracts {
        match contract {
            Contract::OddProduct => {
                let computation_new = computation::constructors::new();
                let computation_odd_product = || computation::messages::odd_product(1000).into();
                runner
                    .prepare_contract(
                        "computation",
                        computation_new,
                        cli.instance_count,
                        computation_odd_product,
                    )
                    .await?;
            }
            Contract::TriangleNumber => {
                let computation_new = computation::constructors::new();
                let computation_triangle_number =
                    || computation::messages::triangle_number(1000).into();
                runner
                    .prepare_contract(
                        "computation",
                        computation_new,
                        cli.instance_count,
                        computation_triangle_number,
                    )
                    .await?;
            }
            Contract::SetFeeTo => {
                let pen_constructor = pendulum_amm::constructors::new(
                    "USDC".to_string(),
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
                    "EUR ".to_string(),
                    "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".to_string(),
                );
                let pen_set_fee_to = || pendulum_amm::messages::set_fee_to(bob.clone()).into();

                runner
                    .prepare_contract(
                        "pendulum_amm",
                        pen_constructor,
                        cli.instance_count,
                        pen_set_fee_to,
                    )
                    .await?;
            }
        }
    }

    let result = runner.run(cli.call_count).await?;

    println!();
    result
        .try_for_each(|block| {
            println!("{}", block.stats);
            future::ready(Ok(()))
        })
        .await?;

    Ok(())
}
