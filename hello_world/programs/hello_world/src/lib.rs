use anchor_lang::prelude::*;

declare_id!("BZwi1Wqyu4hZjf9qg2x21RL2iRm6m5AVZryPfUoZ7qxn");
pub const SEED_PREFIX_SENT: &[u8; 4] = b"sent";
#[program]
pub mod hello_world {
    use super::*;
    use anchor_lang::solana_program;
    // use wormhole_anchor_sdk::wormhole;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
/// Context used to initialize program data (i.e. config).
pub struct Initialize<'info>  {
  #[account(mut)]
  /// Whoever initializes the config will be the owner of the program. Signer
  /// for creating the [`Config`] account and posting a Wormhole message
  /// indicating that the program is alive.
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
  #[account(
    mut,
    seeds = [wormhole::BridgeData::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  // Wormhole bridge data account, aka its config.
  // [`wormhole::post_message`] requires this account be mutable.
  pub wormhole_bridge: Account<'info, wormhole::BridgeData>,
  #[account(
    mut,
    seeds = [wormhole::FeeCollector::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  // Wormhole fee colector if any fees applicable
  // [`wormhole::post_message`] requires this account be mutable.
  pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,
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
    seeds = [wormhole:: SequenceTracker ::SEED_PREFIX, wormhole_emitter.key().as_ref()],
    bump,
    seeds::program = wormhole_program.key(),
  )]
  pub wormhole_sequence: UncheckedAccount<'info>,
  #[account(
    mut,
    seeds = [SEED_PREFIX_SENT, &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..]],
    bump,
  )]
  /// CHECK: Wormhole message account. The Wormhole program writes to this
  /// account, which requires this program's signature.
  pub wormhole_message: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>, // System program.
  pub rent: Sysvar <'info, Rent>,
  pub clock: Sysvar <'info, Clock>,

}

#[account]
pub struct Config {
  pub owner: Pubkey, // Program's owner.
  pub wormhole: Pubkey, // Wormhole program's relevant addresses
  pub batch_id: u32, // nonce
  pub finality: u8, // enum with either `confirmed`or `finalized``
}
impl Config {
  pub const MAXIMUM_SIZE: usize = 8 // dicriminator
    + 32 // owner
    + 32 // wormhole
    + 4 // batch_id
    + 1; // finality

    pub const SEED_PREFIX: &'static [u8; 6] = b"config"; // `b"config"`
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
