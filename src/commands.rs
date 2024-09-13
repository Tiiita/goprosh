use std::process;

use btleplug::platform::Adapter;
use colored::Colorize;
use futures::{executor, TryFutureExt};
use gopro_controller::{self as gopro_lib, GoPro, GoProServices};

fn help_cmd(context: Context) -> Result<(), CommandError> {
    let commands = &context.cmd_service.commands;

    let max_usage_len = commands
        .iter()
        .map(|command| command.usage.len())
        .max()
        .unwrap_or(0);

    println!();
    println!("----------- [{}] -----------", "HELP".yellow().bold());
    for command in commands {
        println!(
            "{:<width$} - {}",
            command.usage,
            command.description,
            width = max_usage_len
        );
    }

    println!();
    Ok(())
}

fn device_cmd<'a>(context: Context<'a>) -> Result<(), CommandError<'a>> {
    if context.args.is_empty() {
        return Err(CommandError::Syntax);
    }

    match context.args[0] {
        "list" => {
            if context.devices.is_empty() {
                return Err(CommandError::ExecutionFailed("No devices connected"));
            }

            println!("{:^15} | {:^12}", "Device Name", "Recording");
            println!("{:-<15}-+-{:-^12}", "", "");
            for gopro in context.devices {
                let recording_icon = if /{ "✅" } else { "❌" };
                println!("{:^15} | {:^12}", gopro.name, recording_icon);
            }
        }

        "add" => {
            let arg = context.args.get(1);

            if arg.is_none() {
                return Err(CommandError::Syntax);
            }

            let arg = arg.unwrap();
    
            let gopros = executor::block_on(gopro_lib::scan(&mut context.gpl_central)).unwrap();
            if !gopros
                .iter()
                .any(|gp| &gp.to_lowercase() == arg)
            {
                return Err(CommandError::ExecutionFailed(
                    "Cannot find gopro with given name",
                ));
            }

            let mut central = executor::block_on(gopro_lib::init(None)).expect("Unable to get adapter");
            let gopro = executor::block_on(gopro_lib::connect(arg.to_string(), &mut central)).expect("Failed to connect");

            context.devices.push(gopro);
        }

        "remove" => {
            println!("Unimplemented");
        }

        "scan" => {
            println!("Scanning, this may take some time..");

            let mut central = executor::block_on(gopro_lib::init(None)).unwrap();
            let gopros = executor::block_on(gopro_lib::scan(&mut central)).unwrap();
            if gopros.is_empty() {
                return Err(CommandError::ExecutionFailed("No nearby gopros found.."));
            } else {
                println!("Found nearby gopros:");
                for ele in gopros {
                    println!("- {}", ele);
                }
            }
        }
        _ => {
            return Err(CommandError::Syntax);
        }
    }

    Ok(())
}

fn record_cmd(_context: Context) -> Result<(), CommandError> {
    Ok(())
}

enum CommandError<'a> {
    Syntax,
    ExecutionFailed(&'a str),
}
pub struct Context<'a> {
    pub name: String,
    pub args: Vec<&'a str>,
    pub devices: &'a mut Vec<GoPro>,
    pub cmd_service: &'a CommandService,
    pub gpl_central: &'a mut Adapter,
}

pub struct Command {
    pub name: String,
    pub description: String,
    pub usage: String,
    executor: Box<dyn Fn(Context) -> Result<(), CommandError>>,
}

impl Command {
    fn new(
        name: &str,
        description: &str,
        usage: &str,
        executor: Box<dyn Fn(Context) -> Result<(), CommandError>>,
    ) -> Self {
        Command {
            name: name.into(),
            usage: usage.into(),
            description: description.into(),
            executor,
        }
    }
}
pub struct CommandService {
    pub commands: Vec<Command>,
}
impl CommandService {
    pub fn new() -> Self {
        CommandService {
            commands: Vec::new(),
        }
    }

    pub fn execute(&self, context: Context) {
        match self.find_by_name(&context.name) {
            Some(cmd) => {
                if let Err(error) = (cmd.executor)(context) {
                    match error {
                        CommandError::ExecutionFailed(msg) => println!("{}", msg.red()),
                        CommandError::Syntax => {
                            println!("{}", format!("Wrong syntax, use: {}", cmd.usage).red())
                        }
                    }
                }
            }
            None => println!(
                "{}",
                "Command not found, use 'help' to list all commands!".red()
            ),
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Command> {
        self.commands
            .iter()
            .find(|cmd| cmd.name.to_lowercase() == name.to_lowercase())
    }
}

pub fn register_commands(service: &mut CommandService) {
    let commands = &mut service.commands;
    commands.push(Command::new(
        "exit",
        "Exits the program",
        "exit",
        Box::new(|_context| {
            println!("Bye.. :)");
            process::exit(0);
        }),
    ));

    commands.push(Command::new(
        "help",
        "List all commands and their usage",
        "help",
        Box::new(help_cmd),
    ));

    commands.push(Command::new(
        "record",
        "Control record status of device(s)",
        "record <start, stop> <device | all>",
        Box::new(record_cmd),
    ));

    commands.push(Command::new(
        "device",
        "Control and list the connected devices or scan for new ones",
        "device <list, add, remove, scan> <device | (all)>",
        Box::new(device_cmd),
    ));
}
