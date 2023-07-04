mod cli;

use std::io::Write;

use cli::{parse_cli_command, process_cli_command, CliError};

use laser_precision_adjust::PrecisionAdjust;
use rustyline_async::ReadlineError;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), std::io::Error> {
    let (mut rl, mut stdout) =
        rustyline_async::Readline::new("> ".to_owned()).expect("Failed to init interactive input!");

    env_logger::builder()
        .format_timestamp(None)
        .parse_default_env()
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Loading config...");
    let config = laser_precision_adjust::Config::load();

    log::info!("{}", config);

    let mut precision_adjust = PrecisionAdjust::with_config(config);

    log::warn!("Testing connections...");
    if let Err(e) = precision_adjust.test_connection().await {
        panic!("Failed to connect to: {:?}", e);
    } else {
        log::info!("Connection successful!");
    }

    let _monitoring = precision_adjust.start_monitoring().await;

    writeln!(stdout, "Type 'help' to see the list of commands!").unwrap();

    loop {
        tokio::select! {
                _ = precision_adjust.print_status(&mut stdout) => { /* show status */ }
                line = rl.readline() => match line {
                Ok(line) => {
                    let line = line.trim();

                    match parse_cli_command(line, &mut stdout) {
                        Ok(cmd) => process_cli_command(&mut precision_adjust, cmd).await,
                        Err(CliError::Parse) => continue,
                        Err(CliError::Exit) | Err(CliError::IO(_)) => {
                            writeln!(stdout, "Exiting...")?;
                            return Ok(());
                        }
                    }
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Closed) => {
                    writeln!(stdout, "Exiting...")?;
                    return Ok(());
                }
                Err(ReadlineError::Interrupted) => {
                    writeln!(stdout, "^C")?;
                    return Ok(());
                }
                Err(ReadlineError::IO(err)) => {
                    writeln!(stdout, "Received err: {:?}", err)?;
                    return Err(err);
                }
            }
        }
    }
}


