
use byteorder::{ByteOrder, BigEndian, LittleEndian};
use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    entrypoint_deprecated,
    entrypoint_deprecated::ProgramResult,
    msg,
    hash::{Hash, HASH_BYTES},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{
        clock::Clock, slot_hashes::SlotHashes, Sysvar,
    },
};

use solana_sdk::program::invoke_signed;
use spl_token::{instruction};
use solana_sdk::program_pack::Pack as TokenPack;
use spl_token::state::{Account as TokenAccount, Mint};

use num_derive::FromPrimitive;
use solana_sdk::{decode_error::DecodeError};
use thiserror::Error;

use crate::{
    error::SolanarollError,
    instruction::SolanarollInstruction,
    // state::Escrow,
    util::{unpack_mint, hash_value, get_slot_hash}
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = SolanarollInstruction::unpack(instruction_data)?;

        match instruction {
            SolanarollInstruction::CommitReveal { reveal_number, amount, roll_under } => {
                msg!("SolanarollInstruction: CommitReveal");
                Self::process_commit_reveal(accounts, amount, reveal_number, roll_under, program_id)
            }
            SolanarollInstruction::ResolveRoll { reveal_number, amount, roll_under } => {
                msg!("SolanarollInstruction: ResolveRoll");
                Self::process_resolve_roll(accounts, amount, reveal_number, roll_under, program_id)
            }
            SolanarollInstruction::Deposit { amount } => {
                msg!("SolanarollInstruction: Deposit");
                Self::process_deposit(accounts, amount, program_id)
            }
            SolanarollInstruction::Withdraw { amount } => {
                msg!("SolanarollInstruction: Withdraw");
                Self::process_withdraw(accounts, amount, program_id)
            }
        }
    }

    pub fn process_deposit(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        let samount = amount.to_string();
        let amount_str: &str = &samount;
        msg!("Amount: ");
        msg!(amount_str);

        if amount <= 0 {
            msg!("Amount is invalid");
            return Err(ProgramError::InvalidAccountData);
        }

        msg!("Getting accounts");

        let accounts_iter = &mut accounts.iter();
        // Set accounts
        let payer_account = next_account_info(accounts_iter)?;
        let fund_account = next_account_info(accounts_iter)?;
        let treasury_token_account = next_account_info(accounts_iter)?;
        let user_token_account = next_account_info(accounts_iter)?;
        let spl_token_program = next_account_info(accounts_iter)?;
        let treasury_account = next_account_info(accounts_iter)?;

        let fund_account_balance = fund_account.lamports();
        let treasury_account_balance = treasury_account.lamports();

        if fund_account_balance <= 0 {
            msg!("Treasury fund account is empty");
            return Err(ProgramError::InvalidAccountData);
        }

        msg!("invoke: spl_token::instruction::mint_to");

        let treasury_mint = unpack_mint(&treasury_token_account.data.borrow())?;

        msg!("Token supply:");
        let supply = treasury_mint.supply;
        let ssupply = supply.to_string();
        let supply_str: &str = &ssupply;
        msg!(supply_str);

        let (mint_address, mint_bump_seed) = Pubkey::find_program_address(&[&payer_account.key.to_bytes(), br"mint"], &spl_token_program.key);
        //
        // let amount_str = amount.to_string();
        // let samount_str: &str = &amount_str;
        // msg!("Fund amount:");
        // msg!(samount_str);

        // Set amount equal to lamports if no supply
        // Otherwise, set pro-rated based on funds/supply
        let mut amount = fund_account_balance;
        let amount_str = amount.to_string();
        let samount_str: &str = &amount_str;
        msg!("Fund amount:");
        msg!(samount_str);

        let treasury_account_balance_str = treasury_account_balance.to_string();
        let streasury_account_balance_str: &str = &treasury_account_balance_str;
        msg!("Treasury_account_balance:");
        msg!(streasury_account_balance_str);

        // Set amount equal to lamports if no supply
        // Otherwise, set pro-rated based on funds/supply
        let mut token_amount= amount;
        if supply > 0 && treasury_account_balance > 0 {
            token_amount = ((amount as f64 / treasury_account_balance as f64) * supply as f64) as u64;
            let token_amount_str = token_amount.to_string();
            let stoken_amount_str: &str = &token_amount_str;
            msg!("Token amount allocated:");
            msg!(stoken_amount_str);
        }

        let mint_to_instr = spl_token::instruction::mint_to(
            &spl_token::ID,
            treasury_token_account.key,
            user_token_account.key,
            payer_account.key,
            &[],
            token_amount,
        )?;

        let account_infos = &[
            treasury_token_account.clone(),
            user_token_account.clone(),
            payer_account.clone(),
            spl_token_program.clone(),
        ];

        let mint_signer_seeds: &[&[_]] = &[
            &payer_account.key.to_bytes(),
            br"mint",
            &[mint_bump_seed],
        ];

        msg!("Minting");

        invoke_signed(
            &mint_to_instr,
            account_infos,
            &[&mint_signer_seeds],
        )?;

        msg!("Mint successful");

        **fund_account.lamports.borrow_mut() -= amount;
        **treasury_account.lamports.borrow_mut() += amount;

        msg!("Deposit successful");

        Ok(())
    }

    pub fn process_withdraw(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        if amount <= 0 {
            msg!("Amount is invalid");
            return Err(ProgramError::InvalidAccountData);
        }

        let accounts_iter = &mut accounts.iter();

        // Set accounts
        let payer_account = next_account_info(accounts_iter)?;
        let treasury_token_account = next_account_info(accounts_iter)?;
        let user_token_account = next_account_info(accounts_iter)?;
        let spl_token_program = next_account_info(accounts_iter)?;
        let treasury_account = next_account_info(accounts_iter)?;

        let treasury_account_balance = treasury_account.lamports();

        let treasury_mint = unpack_mint(&treasury_token_account.data.borrow())?;

        msg!("Token supply:");
        let supply = treasury_mint.supply;
        let ssupply = supply.to_string();
        let supply_str: &str = &ssupply;
        msg!(supply_str);

        let amount_str = amount.to_string();
        let samount_str: &str = &amount_str;
        msg!("Burning token amount:");
        msg!(samount_str);

        let treasury_account_balance_str = treasury_account_balance.to_string();
        let streasury_account_balance_str: &str = &treasury_account_balance_str;
        msg!("Treasury balance:");
        msg!(streasury_account_balance_str);

        let mut sol_amount = amount;
        if supply > 0 && treasury_account_balance > 0 {
            sol_amount = ((amount as f64 / supply as f64) * treasury_account_balance as f64) as u64;
            let sol_amount_str = sol_amount.to_string();
            let ssol_amount_str: &str = &sol_amount_str;
            msg!("Withdraw equivalent in SOL:");
            msg!(ssol_amount_str);
        }

        let mint_to_instr = spl_token::instruction::burn(
            &spl_token::ID,
            user_token_account.key,
            treasury_token_account.key,
            payer_account.key,
            &[],
            amount
        )?;

        let account_infos = &[
            user_token_account.clone(),
            treasury_token_account.clone(),
            spl_token_program.clone(),
            payer_account.clone(),
        ];

        invoke_signed(
            &mint_to_instr,
            account_infos,
            &[],
        )?;

        msg!("Burn successful");

        **treasury_account.lamports.borrow_mut() -= sol_amount;
        **payer_account.lamports.borrow_mut() += sol_amount;

        msg!("Withdraw successful");

        Ok(())
    }

    pub fn process_commit_reveal(
        accounts: &[AccountInfo],
        amount: u64,
        reveal_number: u64,
        roll_under: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // GAME - COMMIT REVEAL NUMBER

        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let game_account = next_account_info(accounts_iter)?;

        if game_account.owner != program_id {
            msg!("SolanaRoll game_account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut data = game_account.try_borrow_mut_data()?;
        let sysvar_account = next_account_info(accounts_iter)?;
        let sysvar_slot_history = next_account_info(accounts_iter)?;
        let fund_account = next_account_info(accounts_iter)?;

        let current_slot = Clock::from_account_info(sysvar_account)?.slot;
        let hashed_reveal = hash_value(reveal_number);

        // save game data
        BigEndian::write_u32(&mut data[0..4], roll_under);
        BigEndian::write_u64(&mut data[4..12], hashed_reveal);
        BigEndian::write_u64(&mut data[12..20], current_slot);

        Ok(())
    }

    pub fn process_resolve_roll(
        accounts: &[AccountInfo],
        amount: u64,
        reveal_number: u64,
        roll_under: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let game_account = next_account_info(accounts_iter)?;
        let mut data = game_account.try_borrow_mut_data()?;
        let sysvar_account = next_account_info(accounts_iter)?;
        let sysvar_slot_history = next_account_info(accounts_iter)?;
        let fund_account = next_account_info(accounts_iter)?;

        // Get the fund balance - stop if not > 0
        let fund_account_balance = fund_account.lamports();

        let treasury_account = next_account_info(accounts_iter)?;
        let user_account = next_account_info(accounts_iter)?;

        // The game_account must be owned by the program in order to modify its data
        let account_balance = game_account.lamports();
        if game_account.owner != program_id {
            msg!("SolanaRoll game_account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        // confirm same reveal number
        let hashed_reveal = hash_value(reveal_number);
        let saved_hashed_reveal = BigEndian::read_u64(&data[4..12]);
        if saved_hashed_reveal == hashed_reveal {
            let current_slot = Clock::from_account_info(sysvar_account)?.slot;
            let saved_slot = BigEndian::read_u64(&data[12..20]);
            if saved_slot < current_slot {

                // Get slot height of saved transaction
                let slot_hashes_data = sysvar_slot_history.try_borrow_data()?;
                let slot_hash = get_slot_hash(&slot_hashes_data, saved_slot);

                // Couldn't find slot_height in recent slots, invalid
                let buf = [0u8; HASH_BYTES];
                if slot_hash == Hash::new(&buf) {
                    **fund_account.lamports.borrow_mut() -= fund_account_balance;
                    **user_account.lamports.borrow_mut() += fund_account_balance;
                    msg!("Block hash invalid, returning funds");
                } else {
                    msg!("Block height and hash valid, obtaining result");

                    let hashed_slot_hash = hash_value(slot_hash);
                    let val = hash_value(hashed_reveal + hashed_slot_hash);
                    let result = (val % 100) + 1;
                    let s: String = result.to_string();
                    let ss: &str = &s;

                    // Save result
                    BigEndian::write_u64(&mut data[20..28], result);

                    let roll_under_32 = BigEndian::read_u32(&data[0..4]);
                    let roll_under_64 = roll_under_32 as u64;

                    let un: String = roll_under_64.to_string();
                    let uns: &str = &un;

                    msg!("Rolling for a number under:");
                    msg!(uns);
                    msg!("You rolled a:");
                    msg!(ss);

                    msg!("    Fund account balance:");
                    let fab: String = fund_account_balance.to_string();
                    let sfab: &str = &fab;
                    msg!(sfab);

                    if fund_account_balance <= 1000 {
                        msg!("Fund Account is Too Low!");
                        return Err(ProgramError::MissingRequiredSignature);
                    }

                    // Get the treasury balance - stop if not > 0
                    let treasury_account_balance = treasury_account.lamports();

                    // TODO: confirm program owns treasury/fund accs

                    let sub_roll_under_64 = roll_under_64 - 1;
                    let num = 100 - sub_roll_under_64;
                    let tmp = ((num as f64 / sub_roll_under_64 as f64) as f64 + (1 as f64)) as f64;
                    let house = (990 as f64 / 1000 as f64) as f64;
                    let winning_ratio = ((tmp * house) - (1 as f64)) as f64;
                    let fund_account_balance_f = fund_account_balance as f64;
                    let winnings = (fund_account_balance_f * winning_ratio) as u64;

                    let winnings_str: String = winnings.to_string();
                    let swinnings_str: &str = &winnings_str;
                    msg!("Potential winnings:");
                    msg!(swinnings_str);

                    // TODO: max profit configurable
                    let treasury_max_profit_f64 = treasury_account_balance as f64 * 0.01;
                    let treasury_max_profit = treasury_max_profit_f64 as u64;
                    let treasury_max_profit_str: String = treasury_max_profit.to_string();
                    let streasury_max_profit_str: &str = &treasury_max_profit_str;
                    msg!("Treasury max profit:");
                    msg!(streasury_max_profit_str);

                    if winnings > treasury_max_profit {
                        **fund_account.lamports.borrow_mut() -= fund_account_balance;
                        **user_account.lamports.borrow_mut() += fund_account_balance;
                        msg!("Potential profit exceeds max profit allowed");
                    } else {
                        if result >= roll_under_64 {
                            msg!("You LOSE! Funds go to treasury");
                            **fund_account.lamports.borrow_mut() -= fund_account_balance;
                            **treasury_account.lamports.borrow_mut() += fund_account_balance;
                            let lose: String = fund_account_balance.to_string();
                            let slose: &str = &lose;
                            msg!(slose);
                        } else {
                            msg!("You WIN! Funds go to user");
                            **fund_account.lamports.borrow_mut() -= fund_account_balance;
                            let win: String = winnings.to_string();
                            let swin: &str = &win;
                            msg!(swin);

                            if winnings < treasury_account_balance {
                                **treasury_account.lamports.borrow_mut() -= winnings;
                                **user_account.lamports.borrow_mut() += fund_account_balance + winnings;
                            } else {
                                **user_account.lamports.borrow_mut() += fund_account_balance;
                                msg!("Treasury not enough for payout, returning funds");
                            }
                        }
                    }
                }
            } else {
                **fund_account.lamports.borrow_mut() -= fund_account_balance;
                **user_account.lamports.borrow_mut() += fund_account_balance;
                // TODO: fee
                msg!("Block height invalid, returning funds");
            }
        } else {
            **fund_account.lamports.borrow_mut() -= fund_account_balance;
            **user_account.lamports.borrow_mut() += fund_account_balance;
            // TODO: fee
            msg!("Reveal number does not match saved reveal number, returning funds");
        }
        Ok(())
    }
}


// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_sdk::clock::Epoch;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u64>()];
        LittleEndian::write_u64(&mut data, 0);
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let instruction_data: Vec<u8> = Vec::new();

        let accounts = vec![account];

        assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 0);
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 1);
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 2);
    }
}

// Required to support msg! in tests
#[cfg(not(target_arch = "bpf"))]
solana_sdk::program_stubs!();
