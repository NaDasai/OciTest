Ociswap ($OCI)
 

Table des matières
1.	Scrypto	2
Basics	2
Basic terms	2
Decimals (PreciseDecimal)	3
Enums	3
Hello example	4
Code review	4
Code and commands	4
Cross-blueprint calls	5
Access Control	6
Code structure	7
Concentrated liquidity	7
OCI’s brother	7
Active liquidity	7
Flexible fees	8
NFT Liquidity tokens	8
Other projects	8
Trader Joe (Liquidity Book)	9
Liquidity Book vs Uniswap V3	10
Bins	10
Liquidity Tokens	12
Liquidity Tracking	12
Individual swap	13
Tick and Bins	15
Concepts to check	16


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
	// Instantiate to be a component.
        pub fn instantiate_hello() -> ComponentAddress {
            // Bucket to hold tokens.
            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .initial_supply(1000);

// Create vault and put Bucket inside. Can't not use a Bucket later in code!

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


Code structure

LP : createPool : sanity check different tokens, order tokens, wrong address check, tickspacing, deploy LP. getPool (token 1, token 2, fee, tickspacing, deployed pool)
recipient ticklower tickupper amount
swap flash collectProtocol
Concentrated liquidity
 

In this part we’ll understand together what’s concentrated liquidity.

Concentrated liquidity is when you choose a price range for your LP (liquidity pool) instead of having an entire price range like in V2. It was introduced with Uniswap V3 and provides capital efficiency.
If the price stays in the range, both LP earn the same amount of trading fees. So we less capital we have same returns (x4000 for 0.1%). We also have less amount at risk.
Even gas fees are said to be almost 30% cheaper.

It’s money working harder by giving rules. Same amount but more liquidity.

Some math:
https://www.youtube.com/watch?v=_asFkMz4zhw&ab_channel=SmartContractProgrammer


OCI’s brother

Active management component: Constantly adjust price ranges to meet active price ranges in order to continue earning trading fees.

Opportunity: Liquidity management systems that can automatically adjust to LP price ranges.

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
Sphere Finance

 

https://uniswapv3.flipsidecrypto.com/
defi-lab.xyz/uniswapv3simulator

 

https://uniswap.org/blog/uniswap-v3#concentrated-liquidity


Trader Joe (Liquidity Book)

Fungible NFT: non fungible withdraw entire position!! Vs only one share of position => more efficient liquidity management.
Surge fees: on top of base fees (volatility) : mitigate permanent loss
+ Products on top of liquidity book.
X * y = k vs constant sum formula (inside bin price constant, V3 price change in ID)
Capture volatility (can manage parameters max so swaps occur) vs base fee
Stake lp ! More returns.

 
(liquidity in bins, each reserves and price (constant so if swaps in a bin price stays constant))
Amount less reserve => in one bin
Amount bigger reserve => occur in multiple bins and price increase each time you use new bin.
Active bin fixes price.
Receive fungible token for each bin



Liquidity Book vs Uniswap V3
Both Liquidity Book and Uniswap V3 are concentrated liquidity AMMs with some subtle differences:
•	Price ranges are discretized into bins instead of ticks
•	Bin steps (or tick sizes) can be more than 1 basis point
•	Bins use constant sum invariant instead of constant product
•	Liquidity is aggregated vertically instead of horizontally
•	Liquidity positions are fungible
•	Liquidity positions are not restricted to uniform distribution across its price range; they can be distributed in any shape desired
•	Swap fees have fixed + variable pricing, which allows the AMM to charge more fees when market experiences high volatility.
Bins
To allow concentrated liquidity, the price curve is discretized into bins, which is similar to the tick concept in Uniswap V3. The main difference however, is that LB uses the constant sum price formula instead of the constant product formula.
What this means is that each bin represents a single price point and the difference between two consecutive bins is the bin step.
Take for example USDC/USDT again. If the current price is $1 and the bin step is 1 basis point (i.e. 0.0001 or 0.01%), then the next consecutive bins up are 1 * 1.0001 = \$1.00011∗1.0001=$1.0001, 1.0001 * 1.0001 = \$1.000200011.0001∗1.0001=$1.00020001, etc. Astute mathematicians will notice that this is the geometric sequence 1.0001^n.
In addition to using a different pricing invariant, bin steps are not restricted to 1 basis point and is a parameter set by the pool creator. Because of this, there can be multiple markets of the same pair but varying only in their bin step. Put differently, given asset XX, asset YY and bin step s, each market is uniquely identified by its tuple (X, Y, s).
Liquidity in each bin is guided by the constant sum price invariant, P \cdot x + y = LP⋅x+y=L, where xx is the quantity of asset XX, yy is the quantity of asset YY, LL is the amount of liquidity in the bin and PP is the price defined by P = \frac{\Delta y}{\Delta x}P=ΔxΔy. This is more easily visualised by the graph below:
 

Taking the smallest possible value of s which is 1 basis point, how many bins there could possibly be, which comes to [(1+s)2−23,(1+s)223).

Liquidity Tokens
LB introduces a new token standard, LBToken, as the receipt token for liquidity positions.
LBToken tracks the amount of liquidity added to each bin for each user in a given pair. For all intensive purposes, it is almost the same as an ERC-1155 token, but without the functions and variables that are related to NFTs. This makes LBToken fungible, which allows vaults/farms to be easily built on top.
Liquidity Tracking
To track liquidity, we use a three level trie in which each node is a 256 bit array represented by a uint256. The bottom level, depth 2, contains 256^3 = 16,777,2162563=16,777,216 slots, which contains exact the maximum possible number of bins, 2^{24}224.
When a bin has liquidity, its slot will contain a 1, otherwise it contains a 0. If it contains a 1, then the corresponding slot in its parent will also contain a 1, and likewise, so will the corresponding slot in its grandparent.
Since we always know which bin is the active bin, using a tree structure allows us to find the next bin to its left or right that has liquidity quickly by tracking a path via its parent.
 
Individual swap
A swap in a liquidity book pair will cross one or many bins inside the pair. Starting from the active bin, it will consume the liquidity of the bin until reaching out the desired amount or emptying the bin. When a bin is empty, liquidity will be taken on the next closest bin, at the exchange rate defined by the bin. This bin then becomes the active bin of the pair.

Unlike Joe V1, several LBPairs with the same tokens can be created, differentiated by the binStep parameter. When asking the router to do a swap, every swap step will be described using {token In, token Out, bin step}. The LBRouter contract is also compatible with Joe V1 pairs. To swap on a V1 pair, binStep must be put to zero.


Liquidity Book strives to give traders more efficient trades, and liquidity providers enhanced efficiency, mitigation against impermanent loss, and maximum composability of their liquidity.
Liquidity Book’s new design hopes to unlock active liquidity provisioning, without compromising the needs of key stakeholders.


Trader Joe is not the only DEX to throw its hat into the concentrated liquidity ring, with Orca offering its own solution on Solana in March, and QuickSwap teaming up with Algebra on Polygon last month. 

“There is added divergence risk (a.k.a impermanent loss) if the positions are not well managed to stay in market range,” the team said. “For less savvy users, we also plan to offer an automated vault that will help users manage their liquidity positions automatically.” 

Trader Joe also told The Defiant it plans to introduce limit order functionality for Joe v2 in the near future.  
Transactions executed on Trader Joe will be routed between both the v1 and v2 platforms to provide traders with the best pricing available moving forward.
Instead of having one pool with unbound price ranges, Liquidity Book has multiple separate bins with different prices that can be used as building blocks for a liquidity position.” 

The Liquidity Book design of Joe v2 separates liquidity pools into “price bins.” While a traditional AMM lumps all assets provided for a specific token pair into a single pool, Trader Joe aggregates distinct pools of bins containing pairs that are segregated by price into a larger market.

While most AMMs host liquidity pools comprising two separate assets, only the bin corresponding to the current market price comprises both assets in a pairing on Joe v2. 
A single asset is provided to bins above the current price, with the second asset provided to bins below the market. Once a particular bin is depleted — meaning that all of one asset has been removed from the pool by traders and only a single asset remains in the bin — the exchange will shift trading to the next bin, also adjusting the asset’s price in the process.
Liquidity providers (LPs) deposit liquidity into discrete price bins, each bin is assigned a specific price and liquidity providers may provide liquidity to multiple bins.
A key change is that all liquidity deposited into bins, are given fungible token receipts. By adding liquidity non-fungible token receipts, its architecture differs from existing concentrated liquidity solutions.
LPs providing liquidity will be given fungible token receipts, thanks to the discrete bin architecture. Fungible token receipts are more composable, this opens up new possibilities for DeFi integrations with other protocols and products. The more composable the design, the bigger DeFi can grow.

Individually, bins act as constant sum pools with their own liquidity reserves, as opposed to existing AMM designs that use a constant product formula. This model uses pool reserves to calculate prices, which often result in traders paying more for less tokens.
In Liquidity Book, the price is derived from an active bin and is constant inside it. As a result, if the trade occurs using reserves from the bin being used for a transaction, the trade will execute with zero slippage.

The price impact occurs when a trade requires a bin change, which happens when reserves in the currently active bin are not enough to fulfill the trade.

https://joecontent.substack.com/p/introducing-liquidity-book
https://avaxholic.com/how-does-liquidity-book-work-what-is-the-difference-with-uniswap-v3/

Tick and Bins
in Uniswap V3, concentrated Liquidity is implemented by partitioning all available pricing space with ticks. Users can select any two ticks and give liquidity in the range between them. 
In contrast, Liquidity Book divides the price range into discrete bins. The liquidity providers then select the bins into which they want to deposit their funds.

let access_rule: AccessRule = rule!(
    require(general_admin.resource_address())
        || require(internal_admin.resource_address())
);
let my_bucket: Bucket = ResourceBuilder::new_fungible()
    .divisibility(DIVISIBILITY_MAXIMUM)
    .metadata("name", "Regulo")
    .metadata("symbol", "REG")
    .metadata(
        "stage",
        "Stage 1 - Fixed supply, may be restricted transfer",
    )
    .updateable_metadata(
        access_rule.clone(), 
        access_rule.clone() 
    )
    .restrict_withdraw(
        access_rule.clone(), 
        access_rule.clone() 
    )
    .mintable(
        access_rule.clone(), 
        access_rule.clone() 
    )
    .initial_supply(100);



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
