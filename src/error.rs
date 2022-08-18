use std::fmt::Display;
use std::io::Write;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Prints the error on the `stderr` and exits with the provided exit code.
///
/// "error: " is displayed before the error message. The "error" is displayed in
/// red and bold if possible.
pub fn exit<T: Display>(err: T, code: i32) -> ! {
    print_error(&err).unwrap_or_else(|_| eprintln!("error: {}", err));
    std::process::exit(code);
}

/// Prints error on the `stderr`.
///
/// "error: " is displayed before the error message. The "error" is displayed in
/// red and bold if possible.
fn print_error<T: Display>(err: &T) -> Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;

    write!(&mut buffer, "error")?;
    buffer.reset()?;
    writeln!(&mut buffer, ": {}", err)?;

    bufwtr.print(&buffer)?;

    Ok(())
}
