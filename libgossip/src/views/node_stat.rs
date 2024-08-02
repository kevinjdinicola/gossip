use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures_lite::StreamExt;
use iroh::net::endpoint::ConnectionType;

use crate::data::PublicKey;
use crate::doc::Node;
use crate::events::{start_with, Starter};

#[derive(uniffi::Record, Debug)]
pub struct NodeStatsData {
    node_id: PublicKey,
    relay_url: String,
    listen_addrs: Vec<String>,
    connections: Vec<ConnectionStats>
}

#[derive(uniffi::Record, Debug)]
struct ConnectionStats {
    node_id: PublicKey,
    relay_info: String,
    conn_type: String,
    addrs: Vec<String>,
    last_received: String,
    has_send_addr: bool
}
#[uniffi::export(with_foreign)]
#[async_trait]
pub trait NodeStatViewModel: Send + Sync + 'static {
    async fn update_stats(&self, stats: NodeStatsData);
}

#[derive(uniffi::Object, Clone)]
pub struct NodeStat {
    node: Node,
    view_model: Arc<dyn NodeStatViewModel>,
    stop: Arc<AtomicBool>,
}

#[async_trait]
impl Starter for NodeStat {
    async fn start(&self) -> Result<(), anyhow::Error> {
        while !self.stop.load(Ordering::Relaxed) {
            match self.generate_stats().await {
                Ok(s) => {
                    self.view_model.update_stats(s).await;
                }
                _ => {}
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        };

        Ok(())
    }
}

impl Drop for NodeStat {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
impl NodeStat {
    pub fn new(view_model: Arc<dyn NodeStatViewModel>, node: Node) -> Self {
        start_with(NodeStat {
            node, view_model, stop: Arc::new(AtomicBool::new(false))
        })
    }

    async fn generate_stats(&self) -> Result<NodeStatsData, anyhow::Error> {
        let node_id = self.node.node_id();
        let node_status = self.node.node().status().await?;
        let relay_url = node_status.addr.info.relay_url.map(|f|f.to_string()).unwrap_or(String::from("no relay"));
        let listen_addrs: Vec<String> = node_status.listen_addrs.iter().map(|a| format!("{a}")).collect();


        let mut stream = self.node.node().connections().await?;
        let mut connections = vec![];
        while let Some(Ok(ni)) = stream.next().await {
            let node_id = ni.node_id.into();
            let relay_info: String = ni.relay_url.as_ref().map(|rui| {

                format!("{} (Latency: {}, LastAlive: {})", rui.relay_url.to_string(),
                        format_duration(rui.latency),
                        format_duration(rui.last_alive))

            }).unwrap_or(String::default());
            let conn_type = match &ni.conn_type {
                ConnectionType::Direct(d) => {
                    format!("Direct ({})", d)
                }
                ConnectionType::Relay(r) => {
                    format!("Relay ({})", r.as_str())
                }
                ConnectionType::Mixed(a, r) => {
                    format!("Mixed ({}, {})", a.to_string(), r.to_string())
                }
                ConnectionType::None => { String::from("none") }
            };
            let addrs: Vec<String> = ni.addrs.iter().map(|a| {
                let lat = format_duration(a.latency);
                let last = format_duration(a.last_payload);
                let addr = a.addr.to_string();
                format!("{addr} (lat: {lat}ms, last_payload: {last}ms)")
            }).collect();

            let last_received = format_duration(ni.last_received());
            let has_send_addr = ni.has_send_address();

            connections.push(ConnectionStats {
                node_id,
                relay_info,
                conn_type,
                addrs,
                last_received,
                has_send_addr
            })
        }

        let stats = NodeStatsData {
            node_id: node_id.into(),
            relay_url,
            listen_addrs,
            connections
        };


        Ok(stats)
    }

}

fn format_duration(dur: Option<Duration>) -> String {
    dur.map(|d|d.as_millis().to_string()).unwrap_or(String::from("∞"))
}
