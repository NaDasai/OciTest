Ociswap ($OCI)
 

1.	Scrypto
Basics	
Basic terms	
Decimals (PreciseDecimal)	
Enums	
Hello example	
Code review	4
Code and commands	
Cross-blueprint calls	
Access Control	5
2.	Concentrated liquidity	
OCI’s brother	
Active liquidity	
Flexible fees	
NFT Liquidity tokens	
Other projects	
Concepts to check	


1.	Scrypto
 

Basics

In Srypto smart contracts are defiened as blueprints and components.
In a package we can have multiple blueprints. Blueprints instances components (active).
Macro blueprint! generates the ABI (need struct (SBOR) and impl). 
We have:
Blueprint.Function() and Component.Method(&Self).
	resim export -abi <Package> <Blueprint_Name> (to check them)


Basic terms
•	Blueprints: the structure of the smart contract is defined, it contains the logic, it does not maintain a state or an address.
•	Component: instantiates a blueprint, now there is an address and a state.
•	Package: a collection of blueprints that are compiled and published as a single unit. It has an address.
•	Component Ownership: Scrypto allows a component to own other components
•	Function: in Scrypto they are static, do not require state, can be called from a blueprint.
•	Method: It is called from the components and must have a reference to itself, it requires state.
•	Resources: they have to be associated with a quantity, they cannot be copied or destroyed by accident. The 'resources' are always in a 'Bucket' or a 'Vault'.
•	Bucket: Temporary or transitory container of the 'resources', it is created in a transaction and destroyed at the end of it.
•	Vault: Persistent container for 'resources' and is stored inside a component. It can be burned in a 'Bucket'.
Each Bucket and Vault only holds resources of the same type.
•	Token: It is a 'resource' with any amount and granularity (decimals)
•	Badge: A badge is not a primitive type: it is a way of referring to a resource that is used primarily for authorization. A badge can be a fungible or non-fungible resource, depending on your use case.
•	Proof: One of the important conventions of using Badges is that, under normal conditions, they are not actually removed from a vault and passed around. Instead, Proof is created and used to prove that an actor has access to that badge. In short, it is proof that a resource is owned. These tests are always associated with a quantity, it cannot be 0.
Proof is created and used to prove that an actor has access to that badge. (Think of it just like flashing a badge in the real world. Whoever you show it to can see that you possess it, and can inspect it, but you’re not actually handing it to them so they can’t hang on to it.)
•	Transaction Manifest: is the Radix way of creating transactions. It makes it possible to compose multiple actions to be executed atomically by describing a sequence of component calls and resource movements between components. In short, full atomic composability is made possible directly in transactions.
•	Accounts: An account in Radix is not just key pairs. Instead, an account is a component, instantiated from a system-provided account model. The account address is the address of that component.
•	Fees: are the XRD that must be paid to execute a transaction. The fees reflect the load that each transaction places on the network, particularly in the areas of how much work it takes to compute the result and how much permanent storage it requires.
Source : https://academiascrypto-com.translate.goog/scrypto/terminos.html?_x_tr_sl=es&_x_tr_tl=en&_x_tr_hl=es&_x_tr_pto=wapp

Decimals (PreciseDecimal)

let a: Decimal = 10.into();
let b: Decimal = dec!(10);
let c: Decimal = dec!("10.333");
let d: Decimal = Decimal::from(20);
let e: Decimal = Decimal::from("20.123444");

No f32 or f64, doesn’t work in distributed ledgers systems. (fractional part in quotes)
Safe types are types that are guaranteed to panic when they overflow. (safe: ensures transaction rejected if overflow)

Enums

To use enum in scrypto:

use sbor::*;

#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}



Hello example

impl Hello {

        pub fn instantiate_hello() -> ComponentAddress {
            
            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .initial_supply(1000);

            Self {
                sample_vault: Vault::with_bucket(my_bucket)
            }
            .instantiate()
            .globalize()
        }

        pub fn free_token(&mut self) -> Bucket {
            info!("My balance is: {} HelloToken. Now giving away a token!", self.sample_vault.amount());
            self.sample_vault.take(1)
        }
    }

Code review

.instantiate() (component)
.globalize() (or be moved to be owned by other component)
sample_vault: Vault::with_bucket(my_bucket) (can’t put different ressources)

Code and commands

(Installed : Radix Transaction Manifest extension for Visual Studio Code)
rustup target add wasm32-unknown-unknown
git clone https://github.com/radixdlt/radixdlt-scrypto.git
cd radixdlt-scrypto
cargo install --path ./simulator
cd .. 
scrypto new-package ociproject
cd ociproject

resim new-account

Account component address: account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr
Public key: 0286b38b51561f1c3bd3da470cf5f08b66a21932ecbc582543e1db15a2da4a3709
Private key: 9c7278cb5e16037e5753e25e97b3a4cd57a0965693c2ccce86b817c112d3d51a
No configuration found on system. will use the above account as default.
(resim reset)

export account=account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr
export pubkey=0286b38b51561f1c3bd3da470cf5f08b66a21932ecbc582543e1db15a2da4a3709
export privkey=9c7278cb5e16037e5753e25e97b3a4cd57a0965693c2ccce86b817c112d3d51a
resim show $account

resim new-simple-badge
NFAddress: resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6:U32#1
Resource: resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6
NFID: U32#1

resim publish . --owner-badge resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6:U32#1
Success! New Package: package_sim1q93n06902dnn4cgl0zt9gfn67unareyhu4jxevq6j6jqv9j239

export pawspackage=package_sim1q95xv77vtant35f84hjv38vcelvz4ky6hk6udyqw2whs269c9s 

// Republish same package IF STRUCT NOT CHANGED
resim publish . --package-address $pawspackage

resim call-function $pawspackage Ocipaws instantiate_ocipaws
export pawscomponent=component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy

resim call-method $pawscomponent paws "10,resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr" "1,resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr"

resim show $pawscomponent 

Cross-blueprint calls
src/lib.rs
// Import the blueprints that are part of the package
mod coffee_machine; (file.rs name)
mod alarm_clock;

src/alarm_clock.rs
use crate::coffee_machine::*;

r#"<STRING_LITERAL>"# is a Rust syntax to define a string literal that may span multiple lines.

let hello: Hello = Hello::instantiate_hello().into();

external_blueprint! and external_component! Macros

Access Control

(Rules)

    // Define the access rules
    let access_rules = AccessRules::new()
        .method("ban_member", rule!(require_any_of(vec![admin_badge_address, moderator_badge_address])), AccessRule::DenyAll)
        .method("destroy", rule!(require(admin_badge_address) && require_amount(dec!(2), moderator_badge_address)), AccessRule::DenyAll)
        .default(AccessRule::AllowAll, AccessRule::DenyAll);

    // Attach the access rules to the component
    component.add_access_check(access_rules);

(Methods)
// Define the access rules which will govern access to this component's methods
let access_rules = AccessRules::new()
  .method("collect_profits", rule!(require(my_admin_badge)), AccessRule::DenyAll)
  .default(AccessRule::AllowAll, AccessRule::DenyAll);

// Apply my rules, and add my component to global address space so it can be called by others
let component_address = my_component.add_access_check(access_rules).globalize();

(Ressource Action : Transient token)
ResourceBuilder::new_non_fungible(NonFungibleIdType::U64)
  .metadata("name","Undepositable token")
  .mintable(rule!(require(admin)), AccessRule::DenyAll)
  .burnable(rule!(require(admin)), AccessRule::DenyAll)
  .restrict_deposit(AccessRule::DenyAll, AccessRule::DenyAll)
  .no_initial_supply();

2.	Concentrated liquidity
 

In this part we’ll understand together what’s concentrated liquidity.

Concentrated liquidity is when you choose a price range for your LP (liquidity pool) instead of having an entire price range like in V2. It was introduced with Uniswap V3 and provides capital efficiency.
If the price stays in the range, both LP earn the same amount of trading fees. So we less capital we have same returns (x4000 for 0.1%). We also have less amount at risk.
Even gas fees are said to be almost 30% cheaper.

It’s money working harder by giving rules. Same amount but more liquidity.

OCI’s brother

Auto managing protocols : The idea is to find the best strategy allocating the liquidity. Like moving liquidity range as the price moves. Limit risks (hedging)
Key : How much of that capital is actually allocated to right part of the market.
Problem : transactions fees and slippage (wait when slippage or volatility are high)
Not all convert to stable?
Active Liquidity Management (ALM)

Active liquidity
When the price moves from LP price range, LP liquidity is removed (to one asset) from pool and stops earning fees.

Flexible fees 
0.05% 0.3% 1% (according to risk willing to take) fee tiers

NFT Liquidity tokens
Ex: USDC-XRD LP token + data price range.

Ociswap V4 can take part of the fees to buy $OCI.

Other projects

Kyberswap
Revert.finance
Orca (SOL)
Kamino (SOL)
Arrakis.finance (SOL) (taking 10-15% from profits)
Balancer (8 assets)
0x Relayer (orders in a server)

Check liquidity book (Trader Joe)
When volatility high have higher fees to overcome impermanent loss.
Pay more the volatility to have lower slippage.

defi-lab.xyz/uniswapv3simulator

Concepts to check

Pricing model
Impermanent loss
Range limit orders: provide single token as liquidity.
TWAP oracle
Rebalance (Structure pools)
Borrow crypto (bearish pool)
Price impact (slippage)
WETH 1:1 (Radix)
Routing: Oci to Eth (Lp: Oci-Routing), go to Radix-Eth.
Ajustment price
Liquid staking
