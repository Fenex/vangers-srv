extern crate num_traits;

use ::clap::Parser;

mod client;
mod game;
mod player;
mod protocol;
mod server;
mod shell;
mod utils;
mod vanject;

use crate::server::Server;
// use crate::shell::*;

#[derive(Parser, Debug)]
#[clap(
    name = "Vangers Server",
    version,
    author
)]
struct Opts {
    #[clap(
        short,
        long,
        default_value = "2197",
        help = "Server port to listening incoming in-game player connections"
    )]
    port: u16,

    // #[clap(short, long, help = "Accept incoming connections from localhost only")]
    // localhost: bool,

    // #[clap(short, long, help = "Enable interactive shell")]
    // shell: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    // let shell = ShellCmd::parse_from(vec!["", "tdest"]);

    // println!("shell is: {:?}", shell);

    // println!("{:?}", s);

    println!("starting server on port {}", opts.port);

    // /* SHELL INPUT COMMANDS */
    // tokio::spawn(async move {
    //     loop {
    //         let mut cmd = String::new();
    //         std::io::stdin().read_line(&mut cmd);
    //         let mut iter = cmd.split_ascii_whitespace();
    //         match iter.next() {
    //             Some("help") => println!("HELP"),
    //             _ => println!("undefined command")
    //         }
    //     }
    // });

    // println!("is localhost only: {:?}", opts.localhost);

    let mut srv = Server::new(opts.port);
    // if opts.shell {
    // srv.enable_shell();
    // }
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
