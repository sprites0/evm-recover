// Using rmp(rust-messagepack), read ~/abci_state.rmp.

use alloy_rlp::Encodable;
use clap::Parser;
use std::{fs::File, io::Write};
use types::{BlockAndReceipts, EvmBlock};

mod types;

#[derive(Parser)]
struct Args {
    /// Path to the evm s3 directory
    ingest_dir: String,
    /// Block number to ingest
    block_number: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let f = args.block_number - (args.block_number % 1_000_000);
    let s = args.block_number - (args.block_number % 1_000);
    let path = format!(
        "{}/{}/{}/{}.rmp.lz4",
        args.ingest_dir, f, s, args.block_number
    );
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut reader = lz4_flex::frame::FrameDecoder::new(reader);
    let blocks: Vec<BlockAndReceipts> = rmp_serde::decode::from_read(&mut reader)?;
    let block = blocks.first().unwrap();

    let EvmBlock::Reth115(block) = &block.block;
    let header = block.header();

    let output = format!("{}.rlp", header.number);
    {
        let mut buf = vec![];
        header.encode(&mut buf);
        let mut file = File::create(&output)?;
        file.write_all(&buf)?;
    }

    let cmd = format!(
        "reth init-state --chain testnet --header {} --header-hash {} --without-evm --total-difficulty 0",
        output, block.hash()
    );
    println!("Execute the command below:");
    println!("{}", cmd);

    Ok(())
}
