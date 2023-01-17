use scrypto::prelude::*;

//const BASE_FACTOR: Decimal = Decimal::from("0.003");

blueprint! {
    struct Ociswap {
        /// The resource address of LP token.
        lp_resource_address: ResourceAddress, //[REMOVE]
        /// LP tokens mint badge.
        lp_mint_badge: Vault,
        /// The fee to apply for every swap
        /// With fee = BASE_FACTOR * craw_step
        fee: Decimal,
        // [TODO] Variable fee will be added later.      
        /// XRD vault.
        xrd_fee: Vault, 
        /// Determinates the price.
        /// With: active_craw = log(price) / log(1 + craw_step) + 2^23
        active_craw : Decimal,
        craw_step : Decimal,       
        /// The reserve for token A and token B
        a_craws: HashMap<Decimal, Vault>,
        a_lp_craws: HashMap<Decimal, ResourceAddress>,
        b_craws: HashMap<Decimal, Vault>,
        b_lp_craws: HashMap<Decimal, ResourceAddress>,

        // [TODO] Do we add both addresses for checks when adding liquidity.
        a_token_address : ResourceAddress,
        b_token_address : ResourceAddress,
    }

impl Ociswap {
        /// Creates a Ociswap component for token pair A/B and returns the component address
        /// along with the initial LP tokens.
        pub fn instantiate_pool(
            a_token_address: ResourceAddress,
            b_token_address: ResourceAddress, // Not a Bucket
            price: Decimal,
            craw_step: Decimal,
        ) -> ComponentAddress {
            
            // Performing the checks to see if this liquidity pool may be created or not.
            assert!(
                (craw_step >= Decimal::zero()) & (craw_step <= dec!("1")), 
                "[Pool Creation]: Fee must be between 0 and 1"
            );
            // We will add enum for fees.

            assert_ne!(
                a_token_address, b_token_address,
                "[Pool Creation]: Liquidity pools may only be created between two different tokens."
            );
            // assert_ne!(
            //     borrow_resource_manager!(a_token_address).resource_type(), ResourceType::NonFungible,
            //     "[Pool Creation]: Both assets must be fungible."
            // );
            // assert_ne!(
            //     borrow_resource_manager!(b_token_address).resource_type(), ResourceType::NonFungible,
            //     "[Pool Creation]: Both assets must be fungible."
            // );
            // At this point, we know that the pool creation can indeed go through.
            

            // Instantiate our LP token and mint an initial supply of them
            let lp_mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .initial_supply(1);

            let lp_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                // .metadata("symbol", pair_name)
                // .metadata("name", lp_id)
                .mintable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .burnable(rule!(require(lp_mint_badge.resource_address())), LOCKED)
                .no_initial_supply();
            
            
            
            let mut a_craws: HashMap<Decimal, Vault> = HashMap::new();
            let mut b_craws: HashMap<Decimal, Vault> = HashMap::new();

            // [TODO] Check with Flo the log.
            //let active_craw = log(price) / log(1.into() + craw_step) + 2.into().powi(23);
            let active_craw = Decimal::from(200) + price;

            // Juste creating the active craw, creating everything would be expensive in gas.
            // [TODO] Do we even need to create them.
            a_craws.insert(active_craw, Vault::new(a_token_address));
            b_craws.insert(active_craw, Vault::new(b_token_address));

            


                
            // Instantiate our Ociswap component
            let ociswap = Self {
                lp_resource_address,
                lp_mint_badge: Vault::with_bucket(lp_mint_badge),
                fee : Decimal::from("0.003") * craw_step, // BASE_FACTOR
                xrd_fee: Vault::new(RADIX_TOKEN),
                active_craw,
                craw_step,
                a_craws,
                b_craws,
                a_lp_craws : HashMap::new(),
                b_lp_craws : HashMap::new(),
                a_token_address,
                b_token_address,
            }
            .instantiate();

            // ociswap.add_access_check(access_rules);

            // Return the new Ociswap component, as well as the initial supply of LP tokens
            ociswap.globalize()
    }



        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
pub fn add_liquidity(
            &mut self,
            mut a_tokens: Bucket,
            mut b_tokens: Bucket,
            price_inf: Decimal,
            price_sup: Decimal,
        ) -> (Bucket, Bucket) { // No remainer

            assert!(
                !a_tokens.is_empty() & !b_tokens.is_empty(), 
                "[Pool Creation]: Can't create a pool from an empty bucket."
            );

            
            // Sorting the buckets and then creating the hashmap of the vaults from the sorted buckets
            // [TODO] Check borrow.
            let mut buckets: (Bucket, Bucket) = if a_tokens.resource_address().to_vec() > b_tokens.resource_address().to_vec() {
                (a_tokens, b_tokens)
            } else {
                (b_tokens, a_tokens)
            };


            let price: Decimal = self.get_price(self.active_craw);
            //[TODO] When to calculate LP

            // LP to mint
            // L = p * x + y
            let supply_to_mint = price * buckets.0.amount() + buckets.1.amount();
            // You get back later: L * reserves / totalL with reserves getBin(ID)

   
            // [TODO] Range = (log(priceSup) - log(priceInf)) / log(1 + binStep)
            //let range = log(price_sup - price_inf) / log(dec!(1) + self.craw_step);
            let range = price_sup - price_inf;
            // [TODO] Round down.


            let mut inf_id = self.get_id(price_inf);
            // [TODO] Round down.

            let b1_per_caw = buckets.0.amount()/(range/2);
            let b2_per_caw = buckets.1.amount()/(range/2);

            // [TODO] Decimal to integer
            let range = 3;
            for i in 1..range {

                // Craws are created when needed.
                if !self.a_craws.contains_key(&inf_id) {
                    // self.a_craws.insert(price, Vault::new(a_tokens.resource_address()));
                    // self.b_craws.insert(price, Vault::new(b_tokens.resource_address()));
                    if inf_id < self.active_craw {
                        self.a_craws.insert(inf_id, Vault::with_bucket(buckets.0.take(b1_per_caw)));
                        // Create LP token for thid ID.
                        let lp_addresss = self.create_lp_token();
                        self.a_lp_craws.insert(inf_id, lp_addresss);
                    }
                    else {
                        self.b_craws.insert(inf_id, Vault::with_bucket(buckets.1.take(b2_per_caw)));
                        // Create LP token for thid ID.
                        let lp_addresss = self.create_lp_token();
                        self.a_lp_craws.insert(inf_id, lp_addresss);
                    }
                }
                // Get Vault for that ID and add token.
                else{
                    if inf_id < self.active_craw {
                        self.a_craws.get_mut(&inf_id).unwrap().put(buckets.0.take(b1_per_caw));
                    }
                    else {
                        self.b_craws.get_mut(&inf_id).unwrap().put(buckets.1.take(b1_per_caw));
                    }
                }

                inf_id += 1;

            }

            let lp_a_resource_address = self.a_lp_craws.get(&inf_id).unwrap();
            let lp_b_resource_address = self.b_lp_craws.get(&inf_id).unwrap();

            // Get the resource manager of the lp tokens
            let lp_a_resource_manager = borrow_resource_manager!(*lp_a_resource_address);
            let lp_b_resource_manager = borrow_resource_manager!(*lp_b_resource_address);

            // Mint LP tokens according to the share the provider is contributing
            let lp_a_tokens = self
                .lp_mint_badge
                .authorize(|| lp_a_resource_manager.mint(supply_to_mint));
            let lp_b_tokens = self
                .lp_mint_badge
                .authorize(|| lp_b_resource_manager.mint(supply_to_mint));

            // Return the LP tokens along with any remainer
            (lp_a_tokens, lp_b_tokens)
            // [TODO] Token for each bin.
            // [TODO] Do we return buckets.
        }

fn create_lp_token(&mut self) -> ResourceAddress {

        let lp_resource_address = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_MAXIMUM)
        // .metadata("symbol", pair_name)
        // .metadata("name", lp_id)
        .mintable(rule!(require(self.lp_mint_badge.resource_address())), LOCKED)
        .burnable(rule!(require(self.lp_mint_badge.resource_address())), LOCKED)
        .no_initial_supply();

        lp_resource_address
    }

fn get_price (&mut self, id : Decimal) -> Decimal {

        let price = id;

        // Calculate price (constant sum)
        // p = (1+ binStep)*(activeBin - 2^23) (1+binstep) ^(activeId - 2**23)
        // let price: Decimal = (dec!(1) + self.craw_step)*(self.active_craw - dec!(2).powi(23));

        price
    }

fn get_id (&mut self, price : Decimal) -> Decimal {
    
    //let id = log(price) / log(1.into() + self.craw_step) + 2.into().powi(23);

    //id
    price
}

pub fn add_specific_liquidity (&mut self, mut tokens: Bucket, id : Decimal) -> Bucket {

        tokens
    }

    }
}

// Trader Joe Docs : https://docs.traderjoexyz.com/concepts/bin-liquidty
// Trader Joe Whitepaper : https://github.com/traderjoe-xyz/LB-Whitepaper/blob/main/Joe%20v2%20Liquidity%20Book%20Whitepaper.pdf
