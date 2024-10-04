use anchor_lang::prelude::*;

declare_id!("8En47DwXiCWdsiDsRZwuEr2W2wSexFhf4mF9eLiiijoH");

#[program]
pub mod solever_leverage {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
