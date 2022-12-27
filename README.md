This project is to learn Scrypto.

Installed : Radix Transaction Manifest extension for Visual Studio Code

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
export package=package_sim1q93n06902dnn4cgl0zt9gfn67unareyhu4jxevq6j6jqv9j239

resim call-function $package Hello instantiate_hello
Transaction Status: COMMITTED SUCCESS
Transaction Fee: 0.034718985 XRD used for execution, 0 XRD used for royalty, 0 XRD in bad debt
Cost Units: 100000000 limit, 330657 consumed, 0.0000001 XRD per cost unit, 5% tip
Logs: 0
Instructions:
├─ CALL_METHOD ComponentAddress("component_sim1qftacppvmr9ezmekxqpq58en0nk954x0a7jv2zz0hc7q8utaxr") "lock_fee" Decimal("100");
├─ CALL_FUNCTION PackageAddress("package_sim1q93n06902dnn4cgl0zt9gfn67unareyhu4jxevq6j6jqv9j239") "Hello" "instantiate_hello";
└─ CALL_METHOD ComponentAddress("account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr") "deposit_batch" Expression("ENTIRE_WORKTOP");
Instruction Outputs:
├─ ()
├─ ComponentAddress("component_sim1qtg2ygvw5zj8y700rldxrwkh6wuk206ffa2gk9zdhxcqqj4t9x")
└─ ()
New Entities: 2
└─ Component: component_sim1qtg2ygvw5zj8y700rldxrwkh6wuk206ffa2gk9zdhxcqqj4t9x
└─ Resource: resource_sim1qqs33nacnn6jv4vt68ddr5vhcmxsf2zfl0g2r6ua6twspwu3yg

export component=component_sim1qtg2ygvw5zj8y700rldxrwkh6wuk206ffa2gk9zdhxcqqj4t9x

resim show $component
Component: component_sim1qtg2ygvw5zj8y700rldxrwkh6wuk206ffa2gk9zdhxcqqj4t9x
Blueprint: { package_address: package_sim1q93n06902dnn4cgl0zt9gfn67unareyhu4jxevq6j6jqv9j239, blueprint_name: "Hello" }
Access Rules
├─ Native(Method(Component(ClaimRoyalty))) => Group("royalty")
└─ Native(Method(Component(SetRoyaltyConfig))) => Group("royalty")
Default: AllowAll
State: Tuple(Vault("dfde89b3dfea937116a8da641f32e8aa3c3c8c0501ae30362bb872f6bd7bf7f104040000"))
Resources:
└─ { amount: 1000, resource address: resource_sim1qqs33nacnn6jv4vt68ddr5vhcmxsf2zfl0g2r6ua6twspwu3yg, name: "HelloToken", symbol: "HT" }

resim call-method $component free_token
Transaction Status: COMMITTED SUCCESS
Transaction Fee: 0.11449704 XRD used for execution, 0 XRD used for royalty, 0 XRD in bad debt
Cost Units: 100000000 limit, 1090448 consumed, 0.0000001 XRD per cost unit, 5% tip
Logs: 1
└─ [INFO ] My balance is: 1000 HelloToken. Now giving away a token!
Instructions:
├─ CALL_METHOD ComponentAddress("component_sim1qftacppvmr9ezmekxqpq58en0nk954x0a7jv2zz0hc7q8utaxr") "lock_fee" Decimal("100");
├─ CALL_METHOD ComponentAddress("component_sim1qtg2ygvw5zj8y700rldxrwkh6wuk206ffa2gk9zdhxcqqj4t9x") "free_token";
└─ CALL_METHOD ComponentAddress("account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr") "deposit_batch" Expression("ENTIRE_WORKTOP");
Instruction Outputs:
├─ ()
├─ Bucket(1025u32)
└─ ()
New Entities: 0

resim show $account
Component: account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr
Blueprint: { package_address: package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe, blueprint_name: "Account" }
Access Rules
├─ Native(Method(Component(SetRoyaltyConfig))) => Group("royalty")
└─ Native(Method(Component(ClaimRoyalty))) => Group("royalty")
Default: AllowAll
├─ ScryptoMethod("deposit_batch") => AccessRule(AllowAll)
├─ ScryptoMethod("deposit") => AccessRule(AllowAll)
└─ ScryptoMethod("balance") => AccessRule(AllowAll)
Default: Protected(ProofRule(Require(StaticNonFungible(NormalResource[00b91737ee8a4de59d49dad40de5560e5754466ac84cf5432ea95d]:2cb29b44dae86833ff4f042a7783b97de6a0a9a0760d4b0f0456))))
State: Tuple(KeyValueStore("e05a83f46fdfaa821eb208fc80d807fa06443ddd6a2d94f556154636c4f1ec3f03040000"))
Key Value Store: AccountComponent[03db9011fc16410308df6244e579ce540f4b41022e68e50fbfe441][224, 90, 131, 244, 111, 223, 170, 130, 30, 178, 8, 252, 128, 216, 7, 250, 6, 68, 61, 221, 106, 45, 148, 245, 86, 21, 70, 54, 196, 241, 236, 63, 3, 4, 0, 0]
├─ ResourceAddress("resource_sim1qqs33nacnn6jv4vt68ddr5vhcmxsf2zfl0g2r6ua6twspwu3yg") => Vault("5e8232f5a0aadfa3353385ca72f5f16811dd377bce02295fd607fc2c36c2872f03040000")
├─ ResourceAddress("resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr") => Vault("e05a83f46fdfaa821eb208fc80d807fa06443ddd6a2d94f556154636c4f1ec3f05040000")
└─ ResourceAddress("resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6") => Vault("0061d73773eeae4a6ffff54513a936bdf27fabc3df3065b329ba9c3e0ce86b3506040000")
Resources:
├─ { amount: 1000, resource address: resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr, name: "Radix", symbol: "XRD" }
├─ { amount: 1, resource address: resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6 }
│  └─ NonFungible { id: NonFungibleId(1u32), immutable_data: Tuple(), mutable_data: Tuple() }
└─ { amount: 1, resource address: resource_sim1qqs33nacnn6jv4vt68ddr5vhcmxsf2zfl0g2r6ua6twspwu3yg, name: "HelloToken", symbol: "HT" }

// Swap 
// Give fees to OCI owners
// Concentrated liquidity?
// Fee? Decimal?
// base_amount

resim call-function $pawspackage Ocipaws instantiate_ocipaws
Transaction Status: COMMITTED SUCCESS
Transaction Fee: 0.046051845 XRD used for execution, 0 XRD used for royalty, 0 XRD in bad debt
Cost Units: 100000000 limit, 438589 consumed, 0.0000001 XRD per cost unit, 5% tip
Logs: 0
Instructions:
├─ CALL_METHOD ComponentAddress("component_sim1qftacppvmr9ezmekxqpq58en0nk954x0a7jv2zz0hc7q8utaxr") "lock_fee" Decimal("100");
├─ CALL_FUNCTION PackageAddress("package_sim1q95xv77vtant35f84hjv38vcelvz4ky6hk6udyqw2whs269c9s") "Ocipaws" "instantiate_ocipaws";
└─ CALL_METHOD ComponentAddress("account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr") "deposit_batch" Expression("ENTIRE_WORKTOP");
Instruction Outputs:
├─ ()
├─ ComponentAddress("component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy")
└─ ()
New Entities: 3
└─ Component: component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy
├─ Resource: resource_sim1qrsfh906ws3hjr5g4h6ar3th5ufpfn0sm6n583xp763saedzmc
└─ Resource: resource_sim1qzt979upwjulnql74w8y7mwn4qcdqmn43qy8y5ghgfjqf9mzru

export pawscomponent=component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy

resim call-method $pawscomponent paws

Transaction Status: COMMITTED FAILURE: ApplicationError(VaultError(ResourceOperationError(ResourceAddressNotMatching)))
Transaction Fee: 0.17540418 XRD used for execution, 0 XRD used for royalty, 0 XRD in bad debt
Cost Units: 100000000 limit, 1670516 consumed, 0.0000001 XRD per cost unit, 5% tip
Logs: 2
├─ [INFO ] My balance is: 1000 TokenA. Now giving swaping a token!
└─ [INFO ] My balance is: 1000 TokenB. Now giving swaping a token!
Instructions:
├─ CALL_METHOD ComponentAddress("component_sim1qftacppvmr9ezmekxqpq58en0nk954x0a7jv2zz0hc7q8utaxr") "lock_fee" Decimal("100");
├─ CALL_METHOD ComponentAddress("component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy") "paws";
└─ CALL_METHOD ComponentAddress("account_sim1q0deqy0uzeqsxzxlvfzw27ww2s85ksgz9e5w2ralu3qsczkapr") "deposit_batch" Expression("ENTIRE_WORKTOP");
New Entities: 0



resim publish . --owner-badge resource_sim1qr4wzvkk33lcpf846rmlfwfez6f339el7y5f7vpnt7pqr59km6:U32#1
export pawspackage=package_sim1q95xv77vtant35f84hjv38vcelvz4ky6hk6udyqw2whs269c9s 

// Republish same package IF STRUCT NOT CHANGED
resim publish . --package-address $pawspackage

resim call-function $pawspackage Ocipaws instantiate_ocipaws
export pawscomponent=component_sim1qtnynf94nlcrpstrc079dz6249jrqfaaxsl3l6agx0rqvcteuy
resim call-method $pawscomponent paws 
resim show $pawscomponent 

// Badge to access?

resim call-method $pawscomponent paws "10,resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr" "1,resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr"