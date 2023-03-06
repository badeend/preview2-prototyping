use crate::{wasi_console, WasiCtx};

#[async_trait::async_trait]
impl wasi_console::WasiConsole for WasiCtx {
    async fn log(
        &mut self,
        level: wasi_console::Level,
        context: String,
        message: String,
    ) -> anyhow::Result<()> {
        println!("{:?} {}: {}", level, context, message);
        Ok(())
    }
}
