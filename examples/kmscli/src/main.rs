use lazy_static::lazy_static;
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo};
use structopt::StructOpt;
use std::net::SocketAddr;
use std::path::PathBuf;

lazy_static! {
    static ref KMS_ADDR: SocketAddr = "127.0.0.1:8080".parse().unwrap();
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    command: Command
}

#[derive(Debug, StructOpt)]
enum Command {
   #[structopt(name = "kms")]
   KMS(KMSOpt)
}

#[derive(Debug, StructOpt)]
struct KMSOpt {
    #[structopt(short = "e", required = true)]
    enclave_info: PathBuf,
}

fn run(args: KMSOpt) {
    println!("[+] Invoke echo function");
    let auditors = vec![];
    let enclave_info = MesateeEnclaveInfo::load(auditors, args.enclave_info.to_str().unwrap()).expect("load");
    let mesatee = Mesatee::new(&enclave_info, "uid1", "token1", *KMS_ADDR).expect("new");
    let response = mesatee.create_key().expect("create_key");
    println!("{:?}", response);
}

fn main() {
    let args = Cli::from_args();
    match args.command {
        Command::KMS(kms_args) => run(kms_args),
    }
    println!("done");
}
