#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::{
    io::{BufRead, Write},
    path::Path,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        todo!("batch files not implemented")
    } else {
        interactive_mode()
    }
}

fn interactive_mode() -> Result<()> {
    const PROMPT: &str = "wish> ";

    let mut input_line = String::new();
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    loop {
        stdout.write_all(PROMPT.as_bytes())?;
        stdout.flush()?;
        stdin.read_line(&mut input_line)?;

        if let Ok(cmds) = parse_line(&input_line) {
            for cmd in cmds {
                if exec_cmd(cmd).is_err() {
                    eprintln!("An error has occured");
                    break;
                };
            }
        } else {
            eprintln!("An error has occured");
        };

        input_line.clear();
    }
}

enum Command<'a> {
    BuiltIn(BuiltInCmd<'a>),
    Run {
        path: &'a Path,
        args: Vec<&'a str>,
        redirect: Option<&'a Path>,
    },
}

enum BuiltInCmd<'a> {
    Exit,
    Cd(&'a Path),
    Pwd,
    Path(Option<Vec<&'a Path>>),
}

fn parse_line(line: &str) -> Result<impl Iterator<Item = Command<'_>>> {
    let mut res = Vec::new();
    let cmds = line.split('&');

    // loop over all commands
    for cmd in cmds {
        res.push(parse_command(cmd)?);
    }

    if res.is_empty() {
        Err(anyhow!(""))
    } else {
        Ok(res.into_iter())
    }
}

fn parse_command(cmd: &str) -> Result<Command<'_>> {
    let mut tokens = cmd.split_whitespace().peekable();
    if let Some(command) = tokens.next() {
        match command {
            "exit" => Ok(Command::BuiltIn(BuiltInCmd::Exit)),
            "cd" => {
                let Some(path) = tokens.next() else {
                    return Err(anyhow!("no path for cd provided"));
                };
                if tokens.peek().is_some() {
                    return Err(anyhow!("too many arguments for cd"));
                }
                Ok(Command::BuiltIn(BuiltInCmd::Cd(Path::new(path))))
            }
            "pwd" => Ok(Command::BuiltIn(BuiltInCmd::Pwd)),
            "path" => {
                let paths: Vec<&Path> = tokens.map(Path::new).collect();

                Ok(Command::BuiltIn(BuiltInCmd::Path(if paths.is_empty() {
                    None
                } else {
                    Some(paths)
                })))
            }
            path => {
                let args: Vec<&str> = tokens.by_ref().take_while(|&token| token != ">").collect();
                let mut redirect = None;

                if let Some(red_op) = tokens.next()
                {
                    assert_eq!(red_op, ">");

                    // skipping redirect operator
                    tokens.next();

                    let Some(red_path) = tokens.next() else {
                        return Err(anyhow!("no path for redirection provided"));
                    };
                    redirect = Some(Path::new(red_path));
                }

                Ok(Command::Run {
                    path: Path::new(path),
                    args,
                    redirect,
                })
            }
        }
    } else {
        Err(anyhow!("no command provided"))
    }
}

fn exec_cmd(cmd: Command) -> Result<()> {
    match cmd {
        Command::BuiltIn(built_in_cmd) => match built_in_cmd {
            BuiltInCmd::Exit => {
                println!("executing exit");
                std::process::exit(0)
            },
            BuiltInCmd::Cd(directory) => {
                println!("executing cd for {}", directory.display());
                std::env::set_current_dir(directory).map_err(Into::into)
            },
            BuiltInCmd::Pwd => {
                println!("{}", std::env::current_dir()?.display());
                Ok(())
            }
            BuiltInCmd::Path(paths) => todo!(),
        },
        Command::Run {
            path,
            args,
            redirect,
        } => todo!(),
    }
}
