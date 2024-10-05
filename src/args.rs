
#[derive(PartialEq, Debug, clap::Parser)]
pub enum Args {
    ClientAndServer {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>
    },
    Server,
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>
    }
}