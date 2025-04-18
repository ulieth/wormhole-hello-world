use anchor_lang::prelude::*;
pub mod message;
pub use message::*;





declare_id!("BZwi1Wqyu4hZjf9qg2x21RL2iRm6m5AVZryPfUoZ7qxn");
pub const SEED_PREFIX_SENT: &[u8; 4] = b"sent";
#[program]
pub mod hello_world {
    use super::*;
    use anchor_lang::solana_program;
    // use wormhole_anchor_sdk::wormhole;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
      let config = &mut ctx.accounts.config;
      // Set the owner of the config (effectively the owner of the program).
      config.owner = ctx.accounts.owner.key();
      // Set Wormhole related addresses.
      {
        let wormhole = &mut config.wormhole;
        // wormhole::BridgeData (Wormhole's program data).
        wormhole.bridge = ctx.accounts.wormhole_bridge.key();
        // wormhole::FeeCollector (lamports collector for posting messages).
        wormhole.fee_collector = ctx.accounts.wormhole_fee_collector.key();
        // wormhole::SequenceTracker (tracks # of messages posted by this program).
        wormhole.sequence = ctx.accounts.wormhole_sequence.key();
      }

      config.batch_id = 0;
      config.finality = wormhole::Finality::Confirmed as u8;
      // The emitter PDA is just a mechanism to have the program sign for the `wormhole::post_message` instruction.
      ctx.accounts.wormhole_emitter.bump = ctx.bumps.wormhole_emitter;
      {
        let fee = ctx.accounts.wormhole_bridge.fee();
        if fee > 0 {
          solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
              &ctx.accounts.owner.key(),
              &ctx.accounts.wormhole_fee_collector.key(),
              fee,
            ),
            &ctx.accounts.to_account_infos(),
          )?;
        }
        // `wormhole::post_message` requires two signers: one for the
        // emitter and another for the wormhole message data. Both of these
        // accounts are owned by this program.
        // To handle the wormhole message data account: generate a PDA.
        let wormhole_emitter = &ctx.accounts.wormhole_emitter;
        let config = &ctx.accounts.config;

        let mut payload: Vec<u8> = Vec::new();
        HelloWorldMessage::serialize(
          &HelloWorldMessage::Alive { program_id: *ctx.program_id },
          &mut payload)?;
        wormhole::post_message(
          CpiContext::new_with_signer(
            ctx.accounts.wormhole_program.to_account_info(),
            wormhole::PostMessage {
              config: ctx.accounts.wormhole_bridge.to_account_info(),
              message: ctx.accounts.wormhole_message.to_account_info(),
              emitter: wormhole_emitter.to_account_info(),
              sequence: ctx.accounts.wormhole_sequence.to_account_info(),
              payer: ctx.accounts.owner.to_account_info(),
              fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
              clock: ctx.accounts.clock.to_account_info(),
              rent: ctx.accounts.rent.to_account_info(),
              system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[&[SEED_PREFIX_SENT, &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..], &[ctx.bumps.wormhole_message]],
            &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]]
            ],
          ),
          config.batch_id,
          payload,
          config.finality.try_into().unwrap(),
        )?;

      }

        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
/// Context used to initialize program data (i.e. config).
pub struct Initialize<'info> {
  /// Whoever initializes the config will be the owner of the program. Signer
  /// for creating the `Config` account and posting a Wormhole message
  /// indicating that the program is alive.
  #[account(mut)]
  pub owner: Signer<'info>,

   /// Config account, which saves program data useful for other instructions.
  #[account(
    init,
    payer = owner,
    seeds = [Config::SEED_PREFIX],
    bump,
    space = Config::MAXIMUM_SIZE,
  )]

  pub config: Account<'info, Config>,

  pub wormhole_program: Program<'info, Wormhole>, // Wormhole program.

  // Wormhole bridge data account, aka its config.
  // [`wormhole::post_message`] requires this account be mutable.
  #[account(
    mut,
    seeds = [wormhole::BridgeData::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

  // Wormhole fee colector if any fees applicable
  // [`wormhole::post_message`] requires this account be mutable.
  #[account(
    mut,
    seeds = [wormhole::FeeCollector::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

  // PDA signer for the `post_message``
  #[account(
    init,
    payer = owner,
    seeds = [WormholeEmitter::SEED_PREFIX],
    bump,
    space = WormholeEmitter::MAXIMUM_SIZE,
  )]
  pub wormhole_emitter: Account<'info, WormholeEmitter>,

  #[account(
    mut,
    seeds = [wormhole::SequenceTracker::SEED_PREFIX, wormhole_emitter.key().as_ref()],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  pub wormhole_sequence: UncheckedAccount<'info>,

  // CHECK: Wormhole message account. The Wormhole program writes to this
  // account, which requires this program's signature.
  #[account(
    mut,
    seeds = [SEED_PREFIX_SENT, &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..]],
    bump,
  )]

  pub wormhole_message: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>, // System program.

  pub rent: Sysvar <'info, Rent>,

  pub clock: Sysvar <'info, Clock>,
}

// Program accounts

#[account]
pub struct Config {
  pub owner: Pubkey, // Program's owner.
  pub wormhole: WormholeAddresses, // Wormhole program's relevant addresses
  pub batch_id: u32, // nonce
  pub finality: u8, // enum with either `confirmed`or `finalized``
}
impl Config {
  pub const MAXIMUM_SIZE: usize = 8 // discriminator
    + 32 // owner
    + 32 // wormhole
    + 4 // batch_id
    + 1; // finality

    pub const SEED_PREFIX: &'static [u8; 6] = b"config"; // `b"config"`
}
#[derive(Default, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
/// Wormhole program related addresses.
pub struct WormholeAddresses {
  pub bridge: Pubkey, // BridgeData address from (wormhole_anchor_sdk::wormhole::BridgeData)
  pub fee_collector: Pubkey, // FeeCollector address from (wormhole_anchor_sdk::wormhole::FeeCollector)
  pub sequence: Pubkey, // SequenceTracker address from (wormhole_anchor_sdk::wormhole::SequenceTracker)
}
impl WormholeAddresses {
  pub const LEN: usize = 32 // confi
    + 32 // fee_collector
    + 32; // sequence
}

#[account]
#[derive(Default)]
pub struct WormholeEmitter {
  pub bump: u8,
}
impl WormholeEmitter {
  pub const MAXIMUM_SIZE: usize = 8 // dicriminator
    + 1; // bump

  pub const SEED_PREFIX: &'static [u8; 7] = b"emitter"; // `b"emitter"`
}
