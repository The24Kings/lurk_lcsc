use std::collections::HashMap;

pub trait Command {
    /// Returns the name of the command (e.g., "help", "broadcast").
    fn name(&self) -> &Box<str>;

    /// Executes the command with the provided arguments.
    fn execute(&self, args: &[String]) -> Result<String, String>;
}

pub struct CommandBuilder {
    name: Box<str>,
    execute_fn: Box<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>,
}

impl CommandBuilder {
    /// Creates a new builder for a command with the given name.
    pub fn new<S: Into<Box<str>>>(name: S) -> Self {
        Self {
            name: name.into(),
            execute_fn: Box::new(|_| Err("No execution logic defined.".to_string())),
        }
    }

    /// Sets the execution logic for the command.
    pub fn on_execute<F>(mut self, execute_fn: F) -> Self
    where
        F: Fn(&[String]) -> Result<String, String> + 'static + Send + Sync,
    {
        self.execute_fn = Box::new(execute_fn);
        self
    }

    /// Builds the command as a dynamic object.
    pub fn build(self) -> Box<dyn Command + Send + Sync> {
        Box::new(BuiltCommand {
            name: self.name,
            execute_fn: self.execute_fn,
        })
    }
}

struct BuiltCommand {
    name: Box<str>,
    execute_fn: Box<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>,
}

impl Command for BuiltCommand {
    fn name(&self) -> &Box<str> {
        &self.name
    }

    fn execute(&self, args: &[String]) -> Result<String, String> {
        (self.execute_fn)(args)
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command + Send + Sync>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Registers a new command.
    pub fn register_command(&mut self, command: Box<dyn Command + Send + Sync>) {
        self.commands.insert(command.name().to_string(), command);
    }

    /// Executes a command by name with the given arguments.
    pub fn execute(&self, name: &str, args: &[String]) -> Result<String, String> {
        if let Some(command) = self.commands.get(name) {
            command.execute(args)
        } else {
            Err(format!("Command '{}' not found.", name))
        }
    }
}

fn main() {
    let mut registry = CommandRegistry::new();

    // Define a "help" command.
    let help_command = CommandBuilder::new("help")
        .on_execute(|args| {
            if args.is_empty() {
                Ok("Available commands: help, greet".to_string())
            } else {
                Ok(format!(
                    "Help for command '{}': No additional information.",
                    args[0]
                ))
            }
        })
        .build();
    registry.register_command(help_command);

    // Define a "greet" command.
    let greet_command = CommandBuilder::new("greet")
        .on_execute(|args| {
            if let Some(name) = args.first() {
                Ok(format!("Hello, {}!", name))
            } else {
                Err("Usage: greet <name>".to_string())
            }
        })
        .build();
    registry.register_command(greet_command);

    // Execute commands.
    let result = registry.execute("help", &[]);
    println!("{}", result.unwrap_or_else(|e| e));

    let result = registry.execute("greet", &["Alice".to_string()]);
    println!("{}", result.unwrap_or_else(|e| e));

    let result = registry.execute("greet", &[]);
    println!("{}", result.unwrap_or_else(|e| e));
}
