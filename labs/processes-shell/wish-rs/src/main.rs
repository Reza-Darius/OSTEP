#![allow(dead_code, static_mut_refs)]

use anyhow::{Context, Result, anyhow};
use std::{
    collections::VecDeque,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, Stdio},
};

static mut PATHS: Vec<PathBuf> = const { Vec::new() };

const ERR_MSG: &str = "An error has occurred";
const PROMPT: &str = "wish> ";

fn main() -> Result<()> {
    add_path("/bin");

    let args: Vec<String> = std::env::args().collect();
    let res = if args.len() > 1 {
        let mut iter = args.into_iter();
        // skipping filename
        iter.next();
        batch_mode(iter)
    } else {
        interactive_mode()
    };
    if res.is_err() {
        eprintln!("{ERR_MSG}");
        // eprintln!("{:?}", res);
    }
    Ok(())
}

fn batch_mode(args: impl Iterator<Item = impl AsRef<str>>) -> Result<()> {
    let mut children = VecDeque::new();

    for arg in args {
        let arg = arg.as_ref();
        let file =
            std::fs::read_to_string(arg).with_context(|| anyhow!("failed to read file {}", arg))?;

        for line in file.lines() {
            let Ok(cmds) = parse_line(line) else {
                eprintln!("{ERR_MSG}");
                continue;
            };

            for cmd in cmds {
                match exec_cmd(cmd) {
                    Ok(Some(child)) => {
                        children.push_back(child);
                    }
                    Err(_) => eprintln!("{ERR_MSG}"),
                    _ => {} // res => eprintln!("exec cmd error {:?}", res),
                }
            }

            while let Some(mut child) = children.pop_front() {
                child.wait()?;
            }
        }
    }
    Ok(())
}

fn interactive_mode() -> Result<()> {
    let mut input_line = String::new();
    let mut stdin = std::io::stdin().lock();
    let mut children = VecDeque::new();

    loop {
        print_prompt(PROMPT);

        stdin.read_line(&mut input_line)?;

        if let Ok(cmds) = parse_line(&input_line) {
            for cmd in cmds {
                match exec_cmd(cmd) {
                    Ok(Some(child)) => children.push_back(child),
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("{ERR_MSG}");
                        // eprintln!("{e}");

                        break;
                    }
                }
            }
        } else {
            eprintln!("{ERR_MSG}");
            continue;
        };

        while let Some(mut child) = children.pop_front() {
            child.wait()?;
        }
        input_line.clear();
        // return Ok(());
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
            "exit" => {
                if tokens.next().is_some() {
                    Err(anyhow!("bad input: arguments for exit"))
                } else {
                    Ok(Command::BuiltIn(BuiltInCmd::Exit))
                }
            }
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
            path if !path.is_empty() => {
                let mut args: Vec<&str> = Vec::new();
                let mut redirect = None;

                while let Some(token) = tokens.next() {
                    if token == ">" {
                        if let Some(path) = tokens.next().map(Path::new) {
                            redirect = Some(path)
                        } else {
                            return Err(anyhow!("bad input: no argument for redirect"));
                        }
                    } else {
                        args.push(token);
                    }
                }

                if tokens.next().is_some() {
                    return Err(anyhow!("bad input: too many arguments for redirect"));
                }

                Ok(Command::Run {
                    path: Path::new(path),
                    args,
                    redirect,
                })
            }
            _ => Err(anyhow!("bad input")),
        }
    } else {
        Err(anyhow!("no command provided"))
    }
}

fn exec_cmd(cmd: Command) -> Result<Option<Child>> {
    match cmd {
        Command::BuiltIn(built_in_cmd) => match built_in_cmd {
            BuiltInCmd::Exit => {
                // println!("executing exit");
                std::process::exit(0)
            }
            BuiltInCmd::Cd(directory) => {
                // println!("executing cd for {}", directory.display());

                // do we need to concat with cwd? doesnt seem to be the case
                std::env::set_current_dir(directory)?;
                Ok(None)
            }
            BuiltInCmd::Pwd => {
                println!("{}", std::env::current_dir()?.display());
                Ok(None)
            }
            BuiltInCmd::Path(paths) => {
                unsafe {
                    PATHS.clear();
                };

                if let Some(paths) = paths {
                    for path in paths {
                        add_path(path);
                    }
                }
                Ok(None)
            }
        },
        Command::Run {
            path,
            args,
            redirect,
        } => {
            // println!(
            //     "running program {}, {:?}, {:?}",
            //     path.display(),
            //     args,
            //     redirect
            // );

            // find program in PATHs
            let path = search_path(path)?;
            // println!("found program: {}", path.display());

            let mut cmd = std::process::Command::new(path);
            cmd.args(args);

            // in case of a redirect, we point the child's stdout to the file
            // otherwise the child's stdout points to the parent's stdout
            if let Some(red) = redirect {
                let file = std::fs::File::create(red)?;
                cmd.stdout(file);
            };

            // spawn child process
            let child = cmd.spawn()?;
            Ok(Some(child))
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

    Err(anyhow!("couldnt find program: {}", program.display()))
}
