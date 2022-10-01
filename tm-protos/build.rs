use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["tendermint/abci/types.proto"], &["."])?;
    Ok(())
}
