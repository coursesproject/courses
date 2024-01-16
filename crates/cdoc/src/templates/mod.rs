use std::io::Write;

use std::io;

pub mod new;

fn err_format(e: anyhow::Error, mut f: impl Write) -> io::Result<()> {
    write!(f, "Error {:?}", e)?;
    e.chain()
        .skip(1)
        .try_for_each(|cause| write!(f, " caused by: {}", cause))?;
    Ok(())
}
