pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
// You've generated a new Solana wallet: 5DtgfGrc5NdKZ6ZXKmB2bGkRTBhhibge7ZSgf1NT5W1n

#[cfg(test)]
mod tests {
    use solana_client::rpc_client::RpcClient;
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_instruction::transfer,
        system_program,
    };
    use solana_sdk::{
        hash::hash,
        message::Message,
        signature::{Keypair, Signer, read_keypair_file},
        transaction::Transaction,
    };
    use std::str::FromStr;
    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn airdrop() {
        const RPC_URL: &str =
            "https://turbine-solanad-4cde.devnet.rpcpool.com/9a9da9cf-6db1-47dc-839a-55aca5
c9c80a";

        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        // we'll establish a connection to Solana devnet using the const we defined above
        let client = RpcClient::new(RPC_URL);
        // We're going to claim 2 devnet SOL tokens (2 billion lamports)
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig);
            }
            Err(err) => {
                println!("Airdrop failed: {}", err);
            }
        }
    }
    #[test]
    fn transfer_sol() {
        const RPC_URL: &str =
            "https://turbine-solanad-4cde.devnet.rpcpool.com/9a9da9cf-6db1-47dc-839a-55aca5
c9c80a";
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let pubkey = keypair.pubkey();

        let message_bytes = b"I verify my Solana Keypair!";
        let sig = keypair.sign_message(message_bytes);
        let sig_hashed = hash(sig.as_ref());
        // Verify the signature using the public key
        match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
            true => println!("Signature verified"),
            false => println!("Verification failed"),
        }

        let to_pubkey = Pubkey::from_str("J5Ej9syX5bJz6yw7eBjnfGscTB41yEo6oXqo8Nnf4pWc").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Entire balance transferred: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn submit_rs() {
        const RPC_URL: &str =
            "https://turbine-solanad-4cde.devnet.rpcpool.com/9a9da9cf-6db1-47dc-839a-55aca5
c9c80a";
        let rpc_client = RpcClient::new(RPC_URL);
        let signer = read_keypair_file("Turbine3-wallet.json").expect("Couldn't find wallet file");
        let mint = Keypair::new();

        let turbin3_prereq_program =
            Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();

        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();

        let mpl_core_program =
            Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        // Define authority PDA (replace seeds/program as needed for your logic)
        let authority_seeds = &[b"collection", collection.as_ref()];
        let (authority, _authority_bump) =
            Pubkey::find_program_address(authority_seeds, &turbin3_prereq_program);

        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),     // user signer
            AccountMeta::new(prereq_pda, false),         // PDA account
            AccountMeta::new(mint.pubkey(), true),       // mint keypair
            AccountMeta::new(collection, false),         // collection
            AccountMeta::new_readonly(authority, false), // authority (PDA)
            AccountMeta::new_readonly(mpl_core_program, false), // mpl core program
            AccountMeta::new_readonly(system_program, false), // system program
        ];

        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer, &mint],
            blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
