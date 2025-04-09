# Hello World

## Objective

Create a `HelloWorld` style example for using Wormhole's generic-messaging layer for developing xDapps (Cross-Chain Decentralized Applications).

## Detailed Design

The HelloWorld example xDapp utilizes the Wormhole generic-messaging layer to send and receive arbitrary messages/payload between smart contracts which for the sake of this example is HelloWorld.

Before the HelloWorld contracts can send and receive messages, the owner of the contract must invoke the `registerEmitter` method to register trusted HelloWorld contracts on other blockchains. The HelloWorld contracts will confirm that all messages that it receives are sent by trusted HelloWorld contracts on other blockchains.

To send a HelloWorld message, one will invoke the `sendMessage` method and pass an arbitrary message as an argument. The HelloWorld contract will then invoke the Wormhole core contract to publish the message. The Wormhole guardians will then attest the message after waiting for the specified number of block confirmations (referred to as `wormholeFinality` in the contracts).

Once the message is attested by the Wormhole guardians, one will invoke the `receiveMessage` method and pass the attested Wormhole message as an argument. The receiving HelloWorld contract will parse and verify the attested Wormhole message, and save the arbitrary HelloWorld message in its state.

To summarise all the Cross program invocations that interact with Wormhole core contract made->
1. **registerEmitter** to flag the user's HelloWorld contract.
2. **sendMessage** invoke the message parsing of wormhole which is picked by Guardians.
3. **receiveMessage** to receive VAAs from the wormhole contract and verify the  payload.
<img width="646" alt="Screenshot 2023-08-19 at 7 41 58 PM" src="https://github.com/wormhole-foundation/wormhole-scaffolding/assets/88841339/03121963-1276-4ee9-baa2-33e2e92a4dbf">


### Solana Interface

```rust
    pub fn initialize(ctx: Context<Initialize>)
    // creates a public function (open for cpi)
    pub fn register_emitter(
        ctx: Context<RegisterEmitter>,
        chain: u16,
        address: [u8; 32],
    )
    // register_emitter can only be invoked by the account owner
    pub fn send_message(ctx: Context<SendMessage>, message: Vec<u8>)
    pub fn receive_message(ctx: Context<ReceiveMessage>, vaa_hash: [u8; 32])
```
