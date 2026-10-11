use std::io::Result;

/// Makes prompts read keys from the console even if stdin is redirected.
/// Prompts read `/dev/tty` by themselves on Unix, but read stdin on Windows,
/// so stdin is replaced with the console input after ADIF is read from it.
#[cfg(windows)]
pub fn ensure_console_input() -> Result<()> {
    use std::{
        fs::OpenOptions,
        io::{Error, IsTerminal, stdin},
        os::windows::io::IntoRawHandle,
    };

    use windows_sys::Win32::System::Console::{STD_INPUT_HANDLE, SetStdHandle};

    if stdin().is_terminal() {
        return Ok(());
    }
    let console = OpenOptions::new().read(true).write(true).open("CONIN$")?;
    let handle = console.into_raw_handle();
    if unsafe { SetStdHandle(STD_INPUT_HANDLE, handle) } == 0 {
        return Err(Error::last_os_error());
    }
    Ok(())
}

/// Makes prompts read keys from the console even if stdin is redirected.
#[cfg(not(windows))]
pub fn ensure_console_input() -> Result<()> {
    Ok(())
}
