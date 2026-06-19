use std::str::FromStr;
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        LegacyTransactionBuilder::new()
    }
}

// Transaction builders
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        Self {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    match args {
        [command] if command == "balance" => Ok(CliCommand::Balance),
        [command, amount, address] if command == "send" => {
            let amount = u64::from_str(amount)
                .map_err(|err| BitcoinError::ParseError(format!("invalid amount: {err}")))?;

            Ok(CliCommand::Send {
                amount,
                address: address.clone(),
            })
        }
        [command, ..] => Err(BitcoinError::ParseError(format!(
            "unknown or invalid command: {command}"
        ))),
        [] => Err(BitcoinError::ParseError("missing command".to_string())),
    }
}

pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Simplified transaction format:
        // version (i32) + input count (u32) + output count (u32) + lock_time (u32)
        const HEADER_LEN: usize = 16;
        if data.len() < HEADER_LEN {
            return Err(BitcoinError::InvalidTransaction);
        }

        let version = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let input_count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let output_count = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let lock_time = u32::from_le_bytes(data[12..16].try_into().unwrap());

        Ok(Self {
            version,
            inputs: Vec::with_capacity(input_count),
            outputs: Vec::with_capacity(output_count),
            lock_time,
        })
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());
        bytes
    }
}
