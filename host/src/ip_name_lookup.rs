#![allow(unused_variables)]

use crate::{
    command::wasi::{ip_name_lookup::{self, ResolveAddressStream}, network::ErrorCode},
    command::wasi::network::{IpAddress, IpAddressFamily, Network},
    command::wasi::poll::Pollable,
    HostResult, WasiCtx,
};

#[async_trait::async_trait]
impl ip_name_lookup::Host for WasiCtx {
    async fn resolve_addresses(
        &mut self,
        network: Network,
        name: String,
        address_family: Option<IpAddressFamily>,
        include_unavailable: bool,
    ) -> HostResult<ResolveAddressStream, ErrorCode> {
        todo!()
    }

    async fn resolve_next_address(
        &mut self,
        stream: ResolveAddressStream,
    ) -> HostResult<Option<IpAddress>, ErrorCode> {
        todo!()
    }

    async fn drop_resolve_address_stream(
        &mut self,
        stream: ResolveAddressStream,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn subscribe(&mut self, stream: ResolveAddressStream) -> anyhow::Result<Pollable> {
        todo!()
    }
}
