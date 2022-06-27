use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "tendermint/crypto/proof.proto",
            "tendermint/crypto/keys.proto",
            "tendermint/version/types.proto",
            "tendermint/types/types.proto",
            "tendermint/abci/types.proto",
        ],
        &["tendermint"],
    )?;
    Ok(())
}
