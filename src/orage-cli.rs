// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT> or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use oragectrl::orage_ctrl_client::OrageCtrlClient;
use oragectrl::{StatusRequest, StatusReply};

use std::sync::Arc;
use std::convert::TryInto;

use clap::{Parser, Subcommand};

pub mod oragectrl {
	tonic::include_proto!("oragectrl");
}

#[derive(Parser, Debug)]
struct Cli {
	/// The port of the connected to orage daemon
	#[clap(short, long, default_value = "20001")]
	port: String,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
	/// Give the status of the orage server
	OrageStatus,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let cli = Cli::parse();

	let mut client = OrageCtrlClient::connect(format!("http://[::1]:{}", cli.port)).await?;

	match cli.command {
		Command::OrageStatus => {
			let request = tonic::Request::new(StatusRequest {});

			let response = client.orage_status(request).await?;

			println!("[ORAGED] requesting status");
		}
	}
	Ok(())
}
