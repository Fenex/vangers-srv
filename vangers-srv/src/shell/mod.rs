use clap::Parser;

#[derive(Parser, Debug)]
pub enum ShellCmd {
    // #[clap(subcommand)]
    Server(Server),
    Game,
    Player,
    Help,
    Exit,
}

#[derive(Parser, Debug)]
enum SubServer {
    Status,
    Shutdown,
    Uptime,
}

#[derive(Parser, Debug)]
pub struct Server {
    #[clap(subcommand)]
    subcmd: SubServer,
    #[clap(long, short)]
    test: bool,
}

// #[derive(Parser, Debug)]
// pub enum SubShellCmd {
//     Server,
//     Game,
//     Player,
//     Help,
// }
