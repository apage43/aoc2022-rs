use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    for line in std::io::stdin().lines() {
        let line = line?;
        // ...
    }
    Ok(())
}
