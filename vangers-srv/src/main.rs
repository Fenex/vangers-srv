extern crate num_traits;

use ::clap::Parser;
use ::tracing_subscriber::EnvFilter;

mod client_id;
mod codec;
mod game;
mod player;
mod protocol;
mod server;
mod service;
mod transport;
mod utils;
mod vanject;

use crate::server::Server;

#[derive(Parser, Debug, Default)]
#[clap(name = "Vangers Server", version, author)]
struct ServerConfig {
    #[clap(
        short,
        long,
        default_value = "2197",
        env = "VANGERS_PORT",
        help = "Server port to listening incoming in-game player connections"
    )]
    pub port: u16,
    #[clap(
        long,
        env = "VANGERS_SUPRESS_LOG_SERVER_TIME",
        help = "Supress log messages for all SERVER_TIME_* events"
    )]
    pub supress_log_server_time: bool,
    #[clap(
        long,
        env = "VANGERS_SUPRESS_LOG_GAMES_LIST_QUERY",
        help = "Supress log messages for all GAMES_LIST_QUERY events"
    )]
    pub supress_log_games_list_query: bool,
    // #[clap(short, long, help = "Accept incoming connections from localhost only")]
    // localhost: bool,

    // #[clap(short, long, help = "Enable interactive shell")]
    // shell: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ::tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let conf: ServerConfig = ServerConfig::parse();

    let srv = Server::new(conf);
    srv.start().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test11() {
        let zevent_size = 0x05_i16.to_le_bytes();
        let zevent_id = 198_u8.to_le_bytes();
        let zresponse = 32607390_i32.to_le_bytes();

        let _ = std::iter::empty()
            .chain(&zevent_size)
            .chain(&zevent_id)
            .chain(&zresponse)
            .map(|&b| b)
            .collect::<Vec<_>>();
    }

    #[test]
    fn test() {
        use std::ffi::CString;

        let hs_in = "Vivat Sicher, Rock'n'Roll forever!!!";

        let c_str = CString::new(hs_in).unwrap();
        let c_str = c_str.into_bytes_with_nul();
        let c_str: Vec<u8> = c_str.into_iter().chain(vec![1, 0, 0].into_iter()).collect();

        if let Some(_pos) = c_str.iter().position(|&e| e == 0) {
            let expected = CString::new(hs_in).unwrap();
            let _expected = expected.to_bytes();

            // println!("expected:\r\n{:?}", expected);
            // println!("c_str is:\r\n{:?}", &c_str[0..pos]);
        }

        // println!("{:?}", c_str);
    }
}
