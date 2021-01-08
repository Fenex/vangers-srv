use clap::Clap;

#[derive(Clap, Debug)]
pub enum ShellCmd {
    // #[clap(subcommand)]
    Server(Server),
    Game,
    Player,
    Help,
    Exit,
}

#[derive(Clap, Debug)]
enum SubServer {
    Status,
    Shutdown,
    Uptime,
}

#[derive(Clap, Debug)]
pub struct Server {
    #[clap(subcommand)]
    subcmd: SubServer,
    #[clap(long, short)]
    test: bool,
}

// #[derive(Clap, Debug)]
// pub enum SubShellCmd {
//     Server,
//     Game,
//     Player,
//     Help,
// }
