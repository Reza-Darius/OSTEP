#![allow(dead_code, static_mut_refs)]

use anyhow::{Result, anyhow};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

static mut PATHS: Vec<PathBuf> = const { Vec::new() };

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
    add_path("/bin");

    let mut input_line = String::new();
    let mut stdin = std::io::stdin().lock();
    // we can hold the lock because stdout is reentrant

    loop {
        print_prompt(PROMPT);

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

fn print_prompt(prompt: &str) {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(prompt.as_bytes()).expect("this cant fail");
    stdout.flush().expect("this cant fail");
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
                let paths = if tokens.peek().is_some() {
                    Some(tokens.map(Path::new).collect())
                } else {
                    None
                };

                Ok(Command::BuiltIn(BuiltInCmd::Path(paths)))
            }
            path => {
                if path.is_empty() {
                    return Err(anyhow!("bad input: empty string"));
                }

                // take_while() consumes the ">" token if its in there
                let args: Vec<&str> = tokens.by_ref().take_while(|&token| token != ">").collect();
                let mut redirect = None;

                if let Some(red_path) = tokens.next() {
                    redirect = Some(Path::new(red_path));
                    if tokens.next().is_some() {
                        return Err(anyhow!("bad input: too many arguments for redirect"));
                    }
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
            }
            BuiltInCmd::Cd(directory) => {
                println!("executing cd for {}", directory.display());
                // do we need to concat with cwd? doesnt seem to be the case
                std::env::set_current_dir(directory).map_err(Into::into)
            }
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
        } => {
            println!(
                "running program {}, {:?}, {:?}",
                path.display(),
                args,
                redirect
            );

            // TODO: handle redirect

            let path = search_path(path)?;
            println!("found program: {}", path.display());

            let mut child = std::process::Command::new(path).args(args).spawn()?;
            child.wait()?;
            Ok(())
        }
    }
}

fn add_path(path: impl Into<PathBuf>) {
    // SAFETY: the shell is single threaded
    unsafe {
        PATHS.push(path.into());
    };
}

fn search_path(program: impl AsRef<Path>) -> Result<PathBuf> {
    let program = program.as_ref();

    // SAFETY: the shell is single threaded
    let paths = unsafe { PATHS.iter() };

    for path in paths {
        let dir_iter = std::fs::read_dir(path)?;
        for entry in dir_iter {
            let Ok(entry) = entry else {
                continue;
            };
            if entry.file_name() == program {
                let path = entry.path();

                return Ok(path);
            }
        }
    }

    Err(anyhow!("couldnt find program"))
}
