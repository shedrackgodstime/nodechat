#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    let mut db_dir = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => {
                if let Some(path) = args.next() {
                    db_dir = Some(std::path::PathBuf::from(path));
                }
            }
            "--help" | "-h" => {
                println!("usage: NodeChat [--db <path>]");
                return Ok(());
            }
            other => {
                eprintln!("unknown argument: {other}");
                println!("usage: NodeChat [--db <path>]");
                return Ok(());
            }
        }
    }

    nodechat::run_app(db_dir)
}
