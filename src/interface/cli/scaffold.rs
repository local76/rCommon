//! CLI Scaffold Router and Command Parser.
//!
//! Part of the Interface (Presentation Layer).
//! Centralizes command-line argument parsing, help screens, version output, and
//! common CLI behaviors.
//!
//! **Taxonomy Classification**: Interface (CLI).

use std::collections::HashMap;
use crate::error::{libraryError, Result};
use super::doctor::run_doctor;

pub use crate::core::{
    is_help_arg, is_version_arg, is_doctor_arg, is_install_arg, is_debug_arg, is_no_color_arg,
    is_json_arg, is_high_contrast_arg, is_accessible_arg, is_tui_arg, is_cli_arg, is_borderless_arg,
};



/// A definition for a command-line option/flag.
#[derive(Debug, Clone)]
pub struct CliOptionDef {
    pub short: char,
    pub long: &'static str,
    pub description: &'static str,
    pub takes_value: bool,
}

/// A definition for a command-line subcommand.
#[derive(Debug, Clone)]
pub struct CliCommandDef {
    pub name: &'static str,
    pub description: &'static str,
}

/// A schema-based parser for CLI arguments.
///
/// Part of the Interface (Presentation Layer).
/// Centralizes command-line argument parsing, help screens, version output, and
/// common CLI behaviors (like running the diagnostic doctor) across rApps.
///
/// # Examples
///
/// ```
/// use library::interface::cli::CliParser;
///
/// let parser = CliParser::new("rfetch", "A fast, modern system fetch utility in Rust.")
///     .logo("====================\n      rFetch\n====================")
///     .command("stdout", "Standard fastfetch stdout print mode")
///     .option('s', "stdout", "Standard fastfetch stdout print mode", false);
///
/// // Standard parsing:
/// let args = vec!["rfetch".to_string(), "stdout".to_string()];
/// let parsed = parser.parse(&args).unwrap();
/// assert_eq!(parsed.command, Some("stdout".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct CliParser {
    pub name: &'static str,
    pub description: &'static str,
    pub commands: Vec<CliCommandDef>,
    pub options: Vec<CliOptionDef>,
    pub logo: Option<&'static str>,
}

/// The result of parsing command-line arguments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedArgs {
    /// The parsed subcommand, if any.
    pub command: Option<String>,
    /// Flags that were present on the command line.
    pub flags: HashMap<String, bool>,
    /// Options that were present on the command line with their values.
    pub options: HashMap<String, String>,
    /// Positional arguments that were not consumed as commands or options.
    pub positional: Vec<String>,
}

impl ParsedArgs {
    /// Check if a flag was passed.
    pub fn has_flag(&self, name: &str) -> bool {
        *self.flags.get(name).unwrap_or(&false)
    }

    /// Retrieve an option value.
    pub fn get_option(&self, name: &str) -> Option<&String> {
        self.options.get(name)
    }
}

/// Action taken by the auto-scaffold router.
/// Classification: Interface (CLI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaffoldAction {
    /// Print help and exit.
    PrintHelp,
    /// Print version and exit.
    PrintVersion,
    /// Run diagnostic doctor and exit.
    RunDoctor,
    /// Continue execution with the parsed arguments.
    Continue(ParsedArgs),
    /// Print error and help, and exit with error.
    Error(libraryError),
}

impl CliParser {
    /// Create a new CLI parser with the given application name and description.
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            commands: Vec::new(),
            options: Vec::new(),
            logo: None,
        }
    }

    /// Set a custom logo/banner to print at the top of the help screen.
    pub fn logo(mut self, logo: &'static str) -> Self {
        self.logo = Some(logo);
        self
    }

    /// Register a subcommand.
    pub fn command(mut self, name: &'static str, description: &'static str) -> Self {
        self.commands.push(CliCommandDef { name, description });
        self
    }

    /// Register an option/flag.
    pub fn option(mut self, short: char, long: &'static str, description: &'static str, takes_value: bool) -> Self {
        self.options.push(CliOptionDef {
            short,
            long,
            description,
            takes_value,
        });
        self
    }

    /// Render the help screen.
    pub fn print_help(&self, logo: Option<&str>) {
        let logo_to_print = logo.or(self.logo);
        if let Some(l) = logo_to_print {
            println!("{}", l);
        }
        println!("{} - {}", self.name, self.description);
        println!("\nUsage:");
        println!("  {} [command] [options]", self.name);

        if !self.commands.is_empty() {
            println!("\nCommands:");
            let max_cmd_len = self.commands.iter().map(|c| c.name.len()).max().unwrap_or(0).max(12);
            for cmd in &self.commands {
                println!("  {:width$} {}", cmd.name, cmd.description, width = max_cmd_len);
            }
        }

        if !self.options.is_empty() {
            println!("\nOptions:");
            let mut opt_strings = Vec::new();
            for opt in &self.options {
                let opt_str = if opt.takes_value {
                    format!("-{}, --{} <value>", opt.short, opt.long)
                } else {
                    format!("-{}, --{}", opt.short, opt.long)
                };
                opt_strings.push((opt_str, opt.description));
            }
            let max_opt_len = opt_strings.iter().map(|(s, _)| s.len()).max().unwrap_or(0).max(24);
            for (opt_str, desc) in opt_strings {
                println!("  {:width$} {}", opt_str, desc, width = max_opt_len);
            }
        }
    }

    /// Automatically parse the environment arguments (`std::env::args()`).
    ///
    /// If `--help`, `-h`, or the subcommand `help` is passed, it prints the help screen (with logo) and exits with code 0.
    /// If `--version`, `-v`, or the subcommand `version` is passed, it prints the version and exits with code 0.
    /// If the subcommand `doctor` is passed, it runs the diagnostic doctor suite and exits with code 0.
    /// If parsing fails, it prints the error and help screen, and exits with code 1.
    pub fn parse_env_args_or_exit(&self, version: &str) -> ParsedArgs {
        let args: Vec<String> = std::env::args().collect();
        self.parse_args_or_exit(&args, version)
    }

    /// Automatically parse the given arguments slice.
    ///
    /// Similar to `parse_env_args_or_exit`, but operates on a provided slice of arguments.
    /// Helpful for testing or custom argument sourcing.
    pub fn parse_args_or_exit(&self, args: &[String], version: &str) -> ParsedArgs {
        match self.determine_scaffold_action(args) {
            ScaffoldAction::PrintHelp => {
                self.print_help(self.logo);
                std::process::exit(0);
            }
            ScaffoldAction::PrintVersion => {
                println!("{} v{}", self.name, version);
                std::process::exit(0);
            }
            ScaffoldAction::RunDoctor => {
                run_doctor();
                std::process::exit(0);
            }
            ScaffoldAction::Continue(parsed) => parsed,
            ScaffoldAction::Error(err) => {
                eprintln!("Error: {}", err);
                self.print_help(self.logo);
                std::process::exit(1);
            }
        }
    }

    /// Determine the scaffold action based on the command line arguments.
    /// This does NOT exit the process, making it fully unit-testable.
    pub fn determine_scaffold_action(&self, args: &[String]) -> ScaffoldAction {
        let sub_args = if args.len() > 1 { &args[1..] } else { &[] };

        // 1. Check for universal help flag/command before full parsing
        if sub_args.iter().any(|a| is_help_arg(a)) {
            return ScaffoldAction::PrintHelp;
        }

        // 2. Check for universal version flag/command before full parsing
        if sub_args.iter().any(|a| is_version_arg(a)) {
            return ScaffoldAction::PrintVersion;
        }

        // 3. Check for doctor command before full parsing
        if sub_args.iter().any(|a| is_doctor_arg(a)) {
            return ScaffoldAction::RunDoctor;
        }

        // 4. Parse using the schema
        match self.parse(args) {
            Ok(parsed) => {
                // Secondary check in case they mapped to options/flags during standard parsing
                if parsed.has_flag("version") || parsed.command.as_deref().is_some_and(is_version_arg) {
                    return ScaffoldAction::PrintVersion;
                }
                if parsed.has_flag("help") || parsed.command.as_deref().is_some_and(is_help_arg) {
                    return ScaffoldAction::PrintHelp;
                }
                if parsed.command.as_deref().is_some_and(is_doctor_arg) {
                    return ScaffoldAction::RunDoctor;
                }
                ScaffoldAction::Continue(parsed)
            }
            Err(err) => ScaffoldAction::Error(err),
        }
    }

    /// Parse the command line arguments.
    pub fn parse(&self, args: &[String]) -> Result<ParsedArgs> {
        let mut parsed = ParsedArgs::default();
        let mut iter = args.iter().skip(1).peekable();

        // Check if first arg is a command
        if let Some(first) = iter.peek() {
            if !first.starts_with('-') {
                let potential_cmd = first.to_string();
                if self.commands.iter().any(|c| c.name == potential_cmd) {
                    parsed.command = Some(potential_cmd);
                    iter.next();
                }
            }
        }

        while let Some(arg) = iter.next() {
            if let Some(long_name) = arg.strip_prefix("--") {
                if let Some(idx) = long_name.find('=') {
                    let key = &long_name[..idx];
                    let val = &long_name[idx + 1..];
                    if let Some(opt) = self.options.iter().find(|o| o.long == key) {
                        if opt.takes_value {
                            parsed.options.insert(key.to_string(), val.to_string());
                        } else {
                            return Err(libraryError::Cli(format!("Option --{} does not take a value", key)));
                        }
                    } else {
                        parsed.options.insert(key.to_string(), val.to_string());
                    }
                } else {
                    if let Some(opt) = self.options.iter().find(|o| o.long == long_name) {
                        if opt.takes_value {
                            if let Some(next_val) = iter.peek() {
                                if !next_val.starts_with('-') {
                                    parsed.options.insert(long_name.to_string(), iter.next().unwrap().clone());
                                } else {
                                    return Err(libraryError::Cli(format!("Option --{} requires a value", long_name)));
                                }
                            } else {
                                return Err(libraryError::Cli(format!("Option --{} requires a value", long_name)));
                            }
                        } else {
                            parsed.flags.insert(long_name.to_string(), true);
                        }
                    } else {
                        parsed.flags.insert(long_name.to_string(), true);
                    }
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                let chars: Vec<char> = arg.chars().skip(1).collect();
                for (char_idx, &c) in chars.iter().enumerate() {
                    if let Some(opt) = self.options.iter().find(|o| o.short == c) {
                        if opt.takes_value {
                            if char_idx == chars.len() - 1 {
                                if let Some(next_val) = iter.peek() {
                                    if !next_val.starts_with('-') {
                                        parsed.options.insert(opt.long.to_string(), iter.next().unwrap().clone());
                                    } else {
                                        return Err(libraryError::Cli(format!("Option -{} requires a value", c)));
                                    }
                                } else {
                                    return Err(libraryError::Cli(format!("Option -{} requires a value", c)));
                                }
                            } else {
                                return Err(libraryError::Cli(format!("Option -{} must be at the end of a flag block to receive a value", c)));
                            }
                        } else {
                            parsed.flags.insert(opt.long.to_string(), true);
                        }
                    } else {
                        parsed.flags.insert(c.to_string(), true);
                    }
                }
            } else {
                parsed.positional.push(arg.clone());
            }
        }

        Ok(parsed)
    }
}

/// **Feature Stub**: This is a fallback placeholder implementation designed to compile successfully and preserve API parity.
/// Returns parsed command or help/version flags.
pub fn parse_cli_args(args: &[String]) -> CliCommand {
    if args.is_empty() {
        return CliCommand::Run;
    }

    // Normalize args: if the first argument starts with '-' or is a known subcommand,
    // treat it as if it's already skipping the executable path.
    let has_exe = !args[0].starts_with('-')
        && !is_doctor_arg(&args[0])
        && !is_version_arg(&args[0])
        && !is_help_arg(&args[0]);

    let args_to_parse = if has_exe {
        args.to_vec()
    } else {
        let mut new_args = vec!["app".to_string()];
        new_args.extend_from_slice(args);
        new_args
    };

    let parser = CliParser::new("app", "app description")
        .command("doctor", "Verify system diagnostics")
        .command("help", "Print help info")
        .command("version", "Print version info")
        .option('h', "help", "Print help info", false)
        .option('v', "version", "Print version info", false);

    match parser.determine_scaffold_action(&args_to_parse) {
        ScaffoldAction::PrintHelp => CliCommand::Help,
        ScaffoldAction::PrintVersion => CliCommand::Version,
        ScaffoldAction::RunDoctor => CliCommand::Doctor,
        ScaffoldAction::Continue(parsed) => {
            if let Some(cmd) = parsed.command {
                CliCommand::Custom(cmd)
            } else {
                CliCommand::Run
            }
        }
        ScaffoldAction::Error(_) => CliCommand::Help,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CliCommand {
    Run,
    Version,
    Help,
    Doctor,
    Custom(String),
}
