Ociswap ($OCI)

Table des matières

1. Scrypto 2
   Basics 2
   Basic terms 2
   Decimals (PreciseDecimal) 3
   Enums 3
   Hello example 4
   Code review 4
   Code and commands 4
   Cross-blueprint calls 5
   Access Control 6
   Code structure 7
2. Concentrated liquidity 7
   OCI’s brother 7
   Active liquidity 8
   Flexible fees 8
   NFT Liquidity tokens 8
   Other projects 8
   Kyberswap 8
   Genesis DEX 8
   Kamino 9
   Trader Joe (Liquidity Book) 11
   Liquidity Book vs Uniswap V3 11
   Tick and Bins 17
   Concepts to check 17

3. Scrypto

Basics

In Srypto smart contracts are defiened as blueprints and components.
In a package we can have multiple blueprints. Blueprints instances components (active).
Macro blueprint! generates the ABI (need struct (SBOR) and impl).
We have:
Blueprint.Function() and Component.Method(&Self).
 resim export -abi <Package> <Blueprint_Name> (to check them)

Basic terms
• Blueprints: the structure of the smart contract is defined, it contains the logic, it does not maintain a state or an address.
• Component: instantiates a blueprint, now there is an address and a state.
• Package: a collection of blueprints that are compiled and published as a single unit. It has an address.
• Component Ownership: Scrypto allows a component to own other components
• Function: in Scrypto they are static, do not require state, can be called from a blueprint.
• Method: It is called from the components and must have a reference to itself, it requires state.
• Resources: they have to be associated with a quantity, they cannot be copied or destroyed by accident. The 'resources' are always in a 'Bucket' or a 'Vault'.
• Bucket: Temporary or transitory container of the 'resources', it is created in a transaction and destroyed at the end of it.
• Vault: Persistent container for 'resources' and is stored inside a component. It can be burned in a 'Bucket'.
Each Bucket and Vault only holds resources of the same type.
• Token: It is a 'resource' with any amount and granularity (decimals)
• Badge: A badge is not a primitive type: it is a way of referring to a resource that is used primarily for authorization. A badge can be a fungible or non-fungible resource, depending on your use case.
• Proof: One of the important conventions of using Badges is that, under normal conditions, they are not actually removed from a vault and passed around. Instead, Proof is created and used to prove that an actor has access to that badge. In short, it is proof that a resource is owned. These tests are always associated with a quantity, it cannot be 0.
Proof is created and used to prove that an actor has access to that badge. (Think of it just like flashing a badge in the real world. Whoever you show it to can see that you possess it, and can inspect it, but you’re not actually handing it to them so they can’t hang on to it.)
• Transaction Manifest: is the Radix way of creating transactions. It makes it possible to compose multiple actions to be executed atomically by describing a sequence of component calls and resource movements between components. In short, full atomic composability is made possible directly in transactions.
• Accounts: An account in Radix is not just key pairs. Instead, an account is a component, instantiated from a system-provided account model. The account address is the address of that component.
• Fees: are the XRD that must be paid to execute a transaction. The fees reflect the load that each transaction places on the network, particularly in the areas of how much work it takes to compute the result and how much permanent storage it requires.
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

use sbor::\*;

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
use crate::coffee_machine::\*;

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
swap flash collectProtocol 2. Concentrated liquidity

In this part we’ll understand together what’s concentrated liquidity.

Previously, liquidity was spread equally throughout the price curve going from zero to infinity, but now it is dispersed within a specified price range.

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
It is similar to Uniswap V3 in a sense that both are tick-based AMMs with customizable price ranges, and use NFTs to represent liquidity positions.
https://blog.kyber.network/kyberswap-elastic-vs-uniswap-v3-a-comparison-7e115117d795

Genesis DEX
Smart swaps. (buy or sell orders that automatically execute when a (user) defined set of criteria is met)
The smart liquidity. Users provide liquidity as with other platforms but instead of having to manually adjust liquidity positions to stay within optimal price ranges, LPs deposit their liquidity, and then simply select a liquidity management strategy. And vault also auto-compounds your rewards

Revert.finance
Orca (SOL)

Kamino (SOL)
On Solana, Kamino can leverage high speeds and low-cost transactions to automatically keep users’ positions within an optimal range for earning fees. Additionally, Kamino auto-compounds fees and rewards into liquidity positions, increasing each LPs stake in concentrated liquidity pools.

Problem:
• CLMM positions must be frequently rebalanced.
• Higher risk of impermanent loss (IL)
• Increased complexity
Kamino has been designed to remove the complexity of managing CLMM positions and does so as efficiently as possible.
When LPs provide liquidity through Kamino:
• The protocol automatically sets and rebalances positions on their behalf.
• Quantitative analysis, provided by a professional quant, determines the parameters of ranges and rebalancing.
• Fees and rewards are auto-compounded back into the liquidity position.
• Users receive a fungible LP token that can be used as collateral on Hubble or in wider DeFi.
In short, Kamino drastically reduces the time and effort required to interact with CLMMs from the backend as an LP.

Arrakis.finance (SOL) (taking 10-15% from profits)
Balancer (8 assets)
An added benefit is that it very easy to create your own pool. Anybody can do it. Just go to “pool management” tab and plug in your wallet, provide the liquidity and you’re good to go. This is good for individuals if there is no pool up to their liking or if they see an opportunity to create pool that will be demanded by others. Projects can also use this feature to attract liquidity.
0x Relayer (orders in a server)
Curve V2
They basically use concentrated liquidity just like Uni V3, but for this AMM rather than concentrating liquidity around price = 1 like they do for stableswaps, it is concentrated around current price. The additional feature is their internal oracle which is used to automatically adjust the range where liquidity is provided rather than having users do it manually like they do on Uni V3.

Bancor
Bancor v2.1 allows single sided exposure and provides impermanent loss insurance. For single sided exposure, Bancor allows LPs to provide only one token rather than forcing them to provide both.
Impermanent loss insurance works like this, if a user deposits $100k of a token into a pool, Bancor matches it with a $100k deposit of BNT. Now both the user and the protocol are accruing fees and rewards. Once the user withdraws liquidity both the user and Bancors LP token gets burned at the same time. The protocol checks to see if the user has suffered any impermanent loss and accordingly distributes the accumulated fees from their LP position to the user.

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

- Products on top of liquidity book.
  X \* y = k vs constant sum formula (inside bin price constant, V3 price change in ID)
  Capture volatility (can manage parameters max so swaps occur) vs base fee
  Stake lp ! More returns.

(liquidity in bins, each reserves and price (constant so if swaps in a bin price stays constant))
Amount less reserve => in one bin
Amount bigger reserve => occur in multiple bins and price increase each time you use new bin.
Active bin fixes price.
Receive fungible token for each bin

Liquidity Book vs Uniswap V3
Both Liquidity Book and Uniswap V3 are concentrated liquidity AMMs with some subtle differences:
• Price ranges are discretized into bins instead of ticks
• Bin steps (or tick sizes) can be more than 1 basis point
• Bins use constant sum invariant instead of constant product
• Liquidity is aggregated vertically instead of horizontally
• Liquidity positions are fungible
• Liquidity positions are not restricted to uniform distribution across its price range; they can be distributed in any shape desired
• Swap fees have fixed + variable pricing, which allows the AMM to charge more fees when market experiences high volatility.
Bins
To allow concentrated liquidity, the price curve is discretized into bins, which is similar to the tick concept in Uniswap V3. The main difference however, is that LB uses the constant sum price formula instead of the constant product formula.
What this means is that each bin represents a single price point and the difference between two consecutive bins is the bin step.
Take for example USDC/USDT again. If the current price is $1 and the bin step is 1 basis point (i.e. 0.0001 or 0.01%), then the next consecutive bins up are 1 _ 1.0001 = \$1.00011∗1.0001=$1.0001, 1.0001 _ 1.0001 = \$1.000200011.0001∗1.0001=$1.00020001, etc. Astute mathematicians will notice that this is the geometric sequence 1.0001^n.
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

To help with this problem, the liquidity book feature in Joe V2 introduced zero or reduced slippage. This is done by offering liquidity providers the ability to select a price range when they deposit crypto into a liquidity pool. The liquidity pair in the pool is then put into bins based on the price range selected by the user. When a trader uses Trader Joe to buy or sell crypto, the liquidity provider will earn a small fraction of the trade when the trade occurs within the price range selected by the liquidity provider and the trader receives zero or reduced slippage rates compared to Joe V1 and other DEX’s.

Trader Joe calls this new method of price selection and asset bin placement “concentrated liquidity,” and it offers three specific benefits. First, liquidity providers will receive more fees with fewer tokens deposited if their selected price range matches the current trading prices for the asset. Second, traders will be able to trade crypto at more predictable prices with zero to little slippage. Finally, traders benefit in a general sense from greater capital efficiency when using the platform.

This is achieved by providing variable fees to liquidity providers and farmers. Fees from trades are the main way liquidity providers earn crypto for providing liquidity into the protocol. To combat impermanent loss, Joe V2 implemented variable fees. The fees collected by liquidity providers on each trade now depend on the volatility of the market and use a tool called the volatility accumulator. When the market is especially volatile, the fees collected increase, and vice versa when the market is less volatile. This is important because the most impermanent loss occurs in volatile market conditions when prices are fluctuating radically. By increasing fees in volatile markets, liquidity providers take on less risk, earn more fees, experience less impermanent loss, and are more likely to keep their position in the protocol, which benefits everyone using the platform.

https://www.influencive.com/trader-joe-launches-new-liquidity-book-feature/

When designing a perfect AMM, one needs to consider the demands of 3 different parties:
• Traders
• Liquidity Providers (LPs)
• Protocol Owners
Traders want zero slippage trades and access to a lot of tokens. LPs want good yield with impermanent loss protection with the option to single sided stake, and they also want the ability to passively manage their LP positions while being assured that their capital is secure on the protocol. Protocol Owners typically want access to a lot of tokens, permissionless creation of liquidity pools, assurance of security, and little to no dependence on token emissions.
Of course, it is impossible to cater to all stakeholders. Some trade-offs need to be made. Therefore, the team at Trader Joe boiled down the priorities for a perfect AMM to focus on zero slippage trades, security, passive liquidity management, and access to a lot of tokens.
The liquidity book is the AMM that checks all of these boxes by retaining the accessibility while fixing some of the inefficiencies of the x\*y=k AMM.

The marquee feature of the liquidity book is their upgraded version of concentrated liquidity called Liquidity Bins.
Liquidity is divided into separate bins with each bin acting as its own constant sum pool using x+y=k rather than x \* y=k. Each bin has its own price range and users can deploy liquidity as they please into any of the bins. However, only one liquidity bin is used at any given time. The smart contract will only use liquidity from the bin that corresponds to current price until all the liquidity from that bin has been depleted and then price moves to the next bin.
Let’s look at an example to better understand this. Suppose the two assets in a trading pair are JOE and USDC. Liquidity can be provided across a variety of bins (i.e. price ranges). If the market price of AVAX is $20 and the bin step is 0.2%, then only liquidity from the $19.99 or $20.03 bin will be used until it’s depleted.

This design allows for zero-slippage trades. The constant sum formula used in each bin means that if a swap uses liquidity only from one bin then there is no price impact, but if there is a large swap causing a change from one bin to another, then there will be some price impact. The other obvious benefit is that it brings a high level of customizability for LP position construction.

Another key difference for the liquidity book is that the LP token receipt is fungible. So rebalancing positions across multiple bins is a lot easier and cheaper. Oftentimes it can also be done within one transaction. This is extremely beneficial for vault strategies that will be constantly monitoring and frequently re-adjusting positions.

https://frogsanon.neworder.network/articles/trader-joe-v2-the-liquidity-book

Although discretized bin liquidity is the standout feature of the liquidity book, it is not the only novel innovation. The Volatility accumulator is a mechanism built to measure market volatility. It essentially measures the amount of bin changes that happen when a swap has been executed. The more bin changes a swap has mademeans the volatility is higher, and higher volatility means the accumulator ramps up. If there are fewer or no bin changes then the accumulator decays back down to zero.

The purpose of collecting this volatility data is to activate a Surge Pricing feature, which is essentially a variable fee that is applied to swaps, in addition to the base fee. Protocols like Uniswap have a standard fee for each pool. But as we know, the majority of the LP positions on Uniswap end up losing money due to impermanent loss because the fees generated do not compensate for the impermanent loss.

With the liquidity book, as the volatility accumulator for a pool spikes up, the variable fee mechanism automatically increases the fees for LPs in that pool and when volatility is low, the fees decrease. The main purpose of this is to protect LPs against impermanent loss. Since most LPs lose money on concentrated liquidity due to either poor management or a poor understanding of impermanent loss, it may dissuade prospective LPs from providing liquidity. With variable fees ensuring minimal impact from impermanent loss, Trader Joe should see an increase in overall liquidity across the protocol.

It's also worth noting that the volatility accumulator doesn’t rely on any external data/feeds. Data is taken directly from the pools so the accumulator updates in real time.

Key Improvements
Implementing concentrated liquidity through liquidity bins rather than tick ranges allows for more capital efficiency and lower price impact on trades. When this is combined with fungible LP tokens which makes readjusting positions easier, the overall liquidity will be much higher which further reduces slippage and price impact.
The volatility accumulator is a novel way to protect against impermanent loss. Rather than paying from the treasury or paying a base fee, a variable fee has a greater likelihood of minimizing impermanent loss for LPs which would likely attract many more LPs to the protocol.

Teams across DeFi started working on potential solutions for a more optimised AMM. This led to a whole wave of innovation in the AMM space pioneered by Uni v3 and their pivot to concentrated liquidity. After that came Curve v2 with their general AMM allowing users to have the same Curve experience but with non-pegged assets. Crocswap came with their own single contract AMM, and Primitive Finance came up with the concept of RMMs.

Liquidity is discretized into bins. Only one bin would contain both tokens (Active bin), all bins on the left contain Token Y while bins on the right contain Token X. If there is a lot of buying up of token X, the bin simply moves to the right and vice versa

Liquidity book aggregates liquidity in each bin (Vertical) rather than horizontally in ranges (Uniswap V3) allowing liquidity to be fungible across all bins.

Contain a base fee and variable fee. The variable fee changes according to how many bins the swap crosses. A large swap will cross many bins resulting in a higher variable fee. This fee is evenly distributed across the bins it crossed.

https://twitter.com/defi_mochi/status/1610593809343336450/photo/1

Crocoswap:
https://crocswap-assets-public.s3.us-east-2.amazonaws.com/CrocSwap_Whitepaper.pdf
By running the entire DEX on one contract you will see major gas & tax savings since tokens aren’t always being reshuffled between contracts.

Muffin
If you want more fee-tiers then you have to make a new pool of the same token which fragments liquidity.
This creation of more choice makes the system more capital-efficient and generates more rewards for LPs.

Cowswap:
Batch auctions are when orders are placed off-chain after which they are aggregated into batches to eventually be settled on-chain. The benefit of batch auctions is that it simplifies the Coincidence of Wants (CoWs).
This means that CoW protocol doesn’t need direct access to on-chain liquidity which gives them significant MEV protection.

The solvers are incentivized to submit the most optimal settlement solution for a designated batch. The solvers are incentivized to compete against each other to find the most optimized solution. The winning solver is then appropriately rewarded with tokens.

Osmosis
Osmosis essentially gives users the tools to create their own AMM pool with maximum customizability. People can set their own parameters for bonding curves, have two token pools or multi-weighted asset pools, they can use any of the already established AMM formulas or make their own one. For LPing the rewards are decided through governance, when coupled with customizability this creates different strategic incentive games.
The longer a LP is locked into a pool the more rewards and governance power they get.
Osmosis creates an “AMM as a serviced infrastructure” model.

Primitive Finance:
https://kaizendao.com/medias/primitive-finance/
https://primitive.mirror.xyz/Audtl29HY_rnhN4E2LwnP7-zjDcDGAyXZ4h3QpDeajg

Something that should be added to improve the trading experience on DEXs is limit orders.

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
Isolated pools

How should Ociswap be

Fee: 0.05% 0.3% 1% (according to risk willing to take) fee tiers
NFT liquidity token: USDC-XRD LP token + data price range.
Compounding: reinvest curve (fees claimed separed?)
Smart swaps. (buy or sell orders that automatically execute when a (user) defined set of criteria is met) (Genesis DEX!) And vault also auto-compounds your rewards.
Pool management: Create your own pool. (Balancer)
Single sided exposure: allows LPs to provide only one token rather than forcing them to provide both. (Bancor v2.1 with BNT)
Surge fees: on top of base fees (volatility accumulator bin change) to mitigate permanent loss. (TJ)
Protocols like Uniswap have a standard fee for each pool. But as we know, the majority of the LP positions on Uniswap end up losing money due to impermanent loss because the fees generated do not compensate for the impermanent loss.
Routing: Transactions executed (on Trader Joe) will be routed between both the v1 and v2 platforms to provide traders with the best pricing available moving forward.
Bins: Asset bin placement, instead of having one pool with unbound price ranges, Liquidity Book has multiple separate bins with different prices that can be used as building blocks for a liquidity position. (TJ)
So rebalancing positions across multiple bins is a lot easier and cheaper. Oftentimes it can also be done within one transaction. This is extremely beneficial for vault strategies that will be constantly monitoring and frequently re-adjusting positions.
Batch auction: (Cowswap) are when orders are placed off-chain after which they are aggregated into batches to eventually be settled on-chain. The benefit of batch auctions is that it simplifies the Coincidence of Wants (CoWs).
This means that CoW protocol doesn’t need direct access to on-chain liquidity which gives them significant MEV protection.

Customizability: Create their own AMM Pool (Osmosis) with maximum customizability, people can set their own parameters for bonding curves, have two token pools or multi-weighted asset pools, they can use any of the already established AMM formulas or make their own one
Governance: The longer a LP is locked into a pool the more rewards and governance power they get. (Osmosis)
Limit orders: +++

Both Liquidity Book and Uniswap V3 are concentrated liquidity AMMs with some subtle differences:
• Price ranges are discretized into bins instead of ticks
• Bin steps (or tick sizes) can be more than 1 basis point
• Bins use constant sum invariant instead of constant product
• Liquidity is aggregated vertically instead of horizontally
• Liquidity positions are fungible
• Liquidity positions are not restricted to uniform distribution across its price range; they can be distributed in any shape desired
• Swap fees have fixed + variable pricing, which allows the AMM to charge more fees when market experiences high volatility.

What this means is that each bin represents a single price point and the difference between two consecutive bins is the bin step.
Take for example USDC/USDT again. If the current price is $1 and the bin step is 1 basis point (i.e. 0.0001 or 0.01%), then the next consecutive bins up are 1 _ 1.0001 = \$1.00011∗1.0001=$1.0001, 1.0001 _ 1.0001 = \$1.000200011.0001∗1.0001=$1.00020001, etc. Astute mathematicians will notice that this is the geometric sequence 1.0001^n.
In addition to using a different pricing invariant, bin steps are not restricted to 1 basis point and is a parameter set by the pool creator. Because of this, there can be multiple markets of the same pair but varying only in their bin step. Put differently, given asset XX, asset YY and bin step s, each market is uniquely identified by its tuple (X, Y, s).

LBToken : liquidity added to each bin.
Bin liquidity: 1 else 0.
LNPair : + binstep
LBRouter: binstep 0 => v1 pairs

Questions:
When to auto-compound to liquidity postion?
Curve v2, internal oracle which is used to automatically adjust the range where liquidity is provided rather than having users do it manually like they do on Uni V3?
Constant sum price formula?
More choices for fee? (Muffin: new pool)


            // l'active bin évolue avec le prix, mais on le set à la création pour commencer à un certain id
            // Prix : activeBin = log(price) / log(1 + binStep) + 2^23
            // Evolution prix : p = (1+ binStep)*(activeBin - 2^23)
            // Pas en fonction des réserves : p = Δy/ Δx
            // range = log(priceSup - priceInf) / log(1 + binStep)
            // LP to mint : L = p * x + y
            // LP to get : L * reserves / totalL

            //p = (1+ binStep)(activeBin - 2^23);

            // the next consecutive bins up are 1 * 1.0001 = \$1.00011∗1.0001=$1.0001, 1.0001 * 1.0001
            // (1+s)^i know i?
            // difference entre active bin et price?
            // activebin ratio obligé.
            // bins crées petit à petit, chaque user a son bin?
            // x ranges importants
            // nombre de bins : range / binstep? ou while < (1+s)^i

            // p = (1+ binStep)**(activeBin - 2**23)
            // activeBin = log(price) / log(1 + binStep) + 2^23
            //Le prix ne dépend pas des reserves
            //p = Δy/ Δx
            // activerbin certain ID?

            // retirer part de bin?



                        //This Decimal type represents a 256 bit fixed-scale decimal number that can have up to 18 decimal places. If you need even more precision,
            //we provide the 512 bit PreciseDecimal type which allows up to 64 decimal places.
            //Represents a signed, bounded fixed-point decimal, where the precision is 10^-18
            //Use PreciseDecimal`if you want to calculate compound interest using the `powi() method as it provides way enough precision to yield to precise results.
            //Represents a 32-byte hash digest. Currently, the only supported hash algorithm is SHA256.
            // keys: BTreeSet<NonFungibleKey> ?
            //use sbor::*;
            
            // https://github.com/radixdlt/scrypto-challenges/blob/main/1-exchanges/RaDEX/src/liquidity_pool.rs
            
            //https://docs.traderjoexyz.com/concepts/bin-math


                        // // Calculate the number of craw
            // let number_craws = 2.into().powi(128); // not quite 0 and infinity but close to it
            
            // let price = active_id - number_craws * craw_step;
            
            // for x in 1..number_craws*2 {
            
            //     let a_craw = Vault::new(a_tokens.resource_address());
            //     let a_craw_address = a_craw.resource_address();
            //     let b_craw = Vault::new(b_tokens.resource_address());
            //     let b_craw_address = b_craw.resource_address();
            
            //     a_craws.insert(price, a_craw);
            //     b_craws.insert(price, b_craw);
                
            //     price = price + craw_step;
            // }

            //             // chercks if price present
            // //self.vaults.contains_key(&address);

            // // gets all prices in vec
            // //self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>();

            // // gets and puts
            // //self.vaults.get_mut(&bucket.resource_address()).unwrap().put(bucket);

            // //
            // //let vault: &mut Vault = self.vaults.get_mut(&resource_address).unwrap();

            // //taking directly amount
            // //self.vaults[&bucket1.resource_address()].amount();

            // // Amount per craw
            // let b1_per_caw = bucket1.amount()/number_craws;
            // let b2_per_caw = bucket2.amount()/number_craws;
            
            // let b1_price = active_id - number_craws * craw_step;
            // let b2_price = active_id;

            // // for x in 1..number_craws {
            // //     let b1_vault = a_craws.get_mut
            // //     craws.insert(b1_price, Vault::with_bucket(bucket1.take(b1_per_caw)));
            // //     b1_price = b1_price + craw_step;

            // //     craws.insert(b2_price, Vault::with_bucket(bucket2.take(b2_per_caw)));
            // //     b2_price = b2_price + craw_step;
            // // }



            // // Find left and right 
            // let (bucket1, bucket2) = if a_tokens.resource_address().to_vec() < b_tokens.resource_address().to_vec() {
            //     (a_tokens, b_tokens)
            //     } else {
            //     (b_tokens, a_tokens)
            //     };

            - à l'intérieur de l'active bin tu peux pas choisir la distribution, c'est selon le ratio entre les 2 tokens
            - ça fait des centaines de tokens si tu as de la liquidité dans une centaine de bin
            - si tu envoies X tokens, tu recevras p*x Y tokens 
            - Tu arrondis à l’entier inférieur (active bin)

            // prevent from adding tokens on sides?
            // comment update sc?
            // withdraw (RessourceAddress, Amount)
            // percentage = LP/totalLp -> token A? token B?
            // remove_liquidity(bucket or vec(bucket or address)?) -> (bucket,bucket)?
            // bimap?
            // RessourceAddress unique?


            // Is it ok to have HashMap::new() at instantiate_pool
            // Fees in XRD or tokens
            // Destruction of tuple without using a part, effect?

            // a and b Bucket don't need to be mut?
            // in borrow, does the address change? b1 b2
            // Ressource unique?

            // [TODO, JOE] for each craw add a totalLP? (L * reserves/ totalL)
            // to burn LP?
            // create_lp_token(&mut self) add name and symbol 

            // JOE check funtions add liquidity, parameters? also remove
            // JOE id of inf and add range?

            // add id to lp name

            // token X, token Y. how do we know? (left, right)

            // Radix feedback?

            // data structure, no duplication, name
            // Vec parameter
            // craw to bin
            // NFT not token, change metadata?
            // V2 how in JOE? binstep 0? range how big?
            // Comment est disperssé après swap tout dans active bin?
            // binstep 0.003 ou 30?
            // reimplement?
            // multi path 
            // protocol on liquidity
            // math function in single sided git
            // check with team about mut a and b
            // todo check https://pro.olympusdao.finance/ and incentives

            // #[scrypto(Debug, TypeId, Encode, Decode, Describe)]
            // !! L -> prix du bin