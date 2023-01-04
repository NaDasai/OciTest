use scrypto::prelude::*;

blueprint! {
    struct Ocipaws {
        // Define what resources and data will be managed by Hello components
        // Vault where Radix Engine stores assets.
        // sample_vault: Vault
        a_vault : Vault,
        b_vault : Vault,
        xrd_vault : Vault,
        fee : Decimal,
        pools : HashMap<ResourceAddress, Decimal>
    }

    impl Ocipaws {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        // Instantiate to be a component.
        pub fn instantiate_ocipaws(fee : Decimal) -> ComponentAddress {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            // Bucket to hold tokens.
            // let my_bucket: Bucket = ResourceBuilder::new_fungible()
            //  //.divisibility(DIVISIBILITY_MAXIMUM)
            //     .metadata("name", "HelloToken")
            //     .metadata("symbol", "HT")
            //     .initial_supply(1000);

            assert!((fee >= Decimal::zero()) & (fee <= dec!("100")), "Fee must be between 0 and 100");

            let my_a_bucket: Bucket = ResourceBuilder::new_fungible()
               .metadata("name", "TokenA")
               .metadata("symbol", "TA")
               .initial_supply(1000);

            let my_b_bucket: Bucket = ResourceBuilder::new_fungible()
              .metadata("name", "TokenB")
              .metadata("symbol", "TB")
              .initial_supply(1000);

                //   let badge: Bucket = ResourceBuilder::new_fungible()
                //      .metadata("name", "Admin badge")
                //      .divisibility(DIVISIBILITY_NONE) // must be before we create the badge
                //      .initial_supply(1);
                
    // // Define the access rules
    // let access_rules = AccessRules::new()
    //     .method("ban_member", rule!(require_any_of(vec![admin_badge_address, moderator_badge_address])), AccessRule::DenyAll)
    //     .method("destroy", rule!(require(admin_badge_address) && require_amount(dec!(2), moderator_badge_address)), AccessRule::DenyAll)
    //     .default(AccessRule::AllowAll, AccessRule::DenyAll);

    // // Attach the access rules to the component
    // component.add_access_check(access_rules);



            let mut my_pool = HashMap::new();
            my_pool.insert(my_a_bucket.resource_address(), 2.into());
            my_pool.insert(my_b_bucket.resource_address(), Decimal::from("0.5"));

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            // Create vault and put Bucket inside. Can't not use a Bucket later in code!
            Self {
                // sample_vault: Vault::with_bucket(my_bucket)
                a_vault : Vault::with_bucket(my_a_bucket),
                b_vault : Vault::with_bucket(my_b_bucket),
                xrd_vault : Vault::new(RADIX_TOKEN),
                fee,
                pools : my_pool
            }
            .instantiate()
            .globalize()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn paws(&mut self, mut swap : Bucket, received_address : ResourceAddress) -> (Bucket, Bucket) {

            // info!("My balance is: {} HelloToken. Now giving away a token!", self.sample_vault.amount());
            info!("My balance is: {} TokenA. Now swapping a token!", self.a_vault.amount());
            info!("My balance is: {} TokenB. Now swapping a token!", self.b_vault.amount());
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            // Sample_vault will handle error if not enough HelloToken.
            // self.sample_vault.take(1)

            //assert_ne!(swap.resource_address(), self.xrd_vault.resource_address(),"Need XRD!");

            self.xrd_vault.put(swap.take(swap.amount())); //  + (swap.amount() * self.fee)

            if received_address == self.a_vault.resource_address() {
                return (self.a_vault.take(swap.amount() * *self.pools.get(&received_address).unwrap()), swap);
            } else if received_address == self.b_vault.resource_address() {
                return (self.b_vault.take(swap.amount() * *self.pools.get(&received_address).unwrap()), swap);
            } else {
                panic!("Address not found!");
            }

        }
    }
}
