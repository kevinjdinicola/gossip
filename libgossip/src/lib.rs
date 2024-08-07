use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use futures_lite::StreamExt;

use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::EnvFilter;
use crate::blob_dispatcher::BlobDataDispatcher;
use crate::data::PublicKey;

use crate::device::DeviceApiServiceProvider;
use crate::doc::{create_or_load_from_fs_reference, Node};
use crate::identity::IdentityService;
use crate::nearby::NearbyService;
use crate::settings::{Service as SettingsService, settings_file_path};
use crate::views::{Global, GlobalViewModel};
use crate::views::nearby_details::{NearbyDetailsViewController, NearbyDetailsViewModel};
use crate::views::node_stat::{NodeStat, NodeStatViewModel};

mod data;
mod device;
mod ble;
mod doc;
mod settings;
mod identity;
mod views;
mod events;
mod nearby;
mod blob_dispatcher;


uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct AppConfig {
    pub data_path: String,
    pub log_directive: Option<String>,
    pub dev_api: Arc<dyn DeviceApiServiceProvider>
}

impl AppConfig {
    pub fn new(data_path: String) -> AppConfig {
        AppConfig {
            data_path,
            log_directive: None,
            dev_api: Arc::new(device::DummyApiServiceProvider {})
        }
    }
}

#[derive(uniffi::Object)]
pub struct AppHost {
    rt: Runtime,
    node: RwLock<Option<Node>>,
    config: AppConfig,
    reset_flag: AtomicBool,
    // services
    settings: SettingsService,
    identity: IdentityService,
    nearby: NearbyService
}

impl AppHost {
    pub fn node(&self) -> Node {
        let lock = self.node.try_read().unwrap();
        lock.as_ref().expect("Couldn't borrow node").clone()
    }
}

#[uniffi::export]
impl AppHost {
    #[uniffi::constructor]
    pub fn new(config: AppConfig) -> AppHost {
        println!("Started libgossip app");
        let directive: &str = config.log_directive.as_ref().map_or("ghostlib=debug", |s| s);

        let filter = EnvFilter::from_default_env()
            .add_directive(directive.parse().unwrap());

        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(filter)
            .with_ansi(false)
            .init();

        let rt = Runtime::new().expect("Unable to start a tokio runtime");
        let h = rt.handle().clone();

        let ah = h.block_on(async move {
            let node = Node::persistent(Path::new(&config.data_path.as_str())).await.unwrap().spawn().await.unwrap();
            let root_doc = create_or_load_from_fs_reference(&node, settings_file_path(&config.data_path)).await;

            println!("default AUTHOR {}", node.authors().default().await.expect(""));
            let settings = SettingsService::new(root_doc);
            let identity = IdentityService::new(settings.identity_doc().await.clone());
            let nearby = NearbyService::new(node.clone(), identity.clone(), settings.clone(), config.dev_api.clone());

            AppHost {
                rt,
                node: RwLock::new(Some(node)),
                config,
                reset_flag: AtomicBool::new(false),

                settings,
                identity,
                nearby
            }
        });
        ah
    }

    pub fn global(&self, view_model: Arc<dyn GlobalViewModel>) -> Arc<Global> {
        let _g = self.rt.enter();
        Global::new(view_model, self.identity.clone(), self.nearby.clone(), self.settings.clone())
    }

    pub fn nearby_details(&self, view_model: Arc<dyn NearbyDetailsViewModel>, subject_pk: PublicKey) -> Arc<NearbyDetailsViewController> {
        let _g = self.rt.enter();
        NearbyDetailsViewController::new(subject_pk, view_model, self.identity.clone(), self.nearby.clone(), self.settings.clone())
    }

    pub fn blobs(&self) -> Arc<BlobDataDispatcher> {
        let _g = self.rt.enter();
        Arc::new(BlobDataDispatcher::new(self.node()))
    }

    pub fn node_stats(&self, view_model: Arc<dyn NodeStatViewModel>) -> Arc<NodeStat> {
        let _g = self.rt.enter();
        Arc::new(NodeStat::new(view_model, self.node()))
    }

    pub fn print_stats(&self) {
        let node = self.node();
        let x = self.rt.block_on(async {
            // let x = node.stats().await?;
            // println!("wtf is {:?}", x);
            let mut cnx = node.connections().await?;
            while let Some(Ok(c)) = cnx.next().await {
                println!("nodeinfo {:?}", c);
                let ci = node.connection_info(c.node_id).await?;
                if let Some(ci) = ci {
                    println!("wtf is this {:?}", ci);
                }
            }
            Ok::<(),anyhow::Error>(())
        });
        if let Err(e) = x {
            println!("ERR {}", e);
        }
    }


    pub fn set_reset_flag(&self) {
        self.reset_flag.store(true, Relaxed);
    }

    pub fn shutdown(&self) {
        let lock_ref = &self.node;
        self.rt.block_on(async {
            // todo shutdown other shit first

            let mut lock = lock_ref.write().await;
            let node: Node = lock.take().unwrap();
            node.shutdown().await.expect("Node shutdown failed");
        });

        if self.reset_flag.load(Relaxed) {
            info!("Deleting all data");
            fs::remove_dir_all(self.config.data_path.as_str()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use futures_util::StreamExt;
    use tokio::runtime::Runtime;
    use tokio::sync::{Mutex};

    use crate::{AppConfig, AppHost};
    use crate::data::BlobHash;
    use crate::doc::Doc;
    use crate::nearby::model::{DebugState, DisplayMessage, NearbyProfile, Status};
    use crate::views::GlobalViewModel;

    const TEST_DIR: &str = "./testtmp";
    fn wipe_test_dir(dir: Option<&str>) {
        let dir: &str = dir.unwrap_or_else(|| TEST_DIR);
        if let Ok(md) = fs::metadata(dir) {
            if md.is_dir() {
                fs::remove_dir_all(dir).unwrap();
            }
        }
    }

    #[test]
    fn will_delete_all_data() {
        wipe_test_dir(None);

        fs::create_dir(TEST_DIR).unwrap();
        let ah = AppHost::new(AppConfig::new(TEST_DIR.into()) );
        ah.set_reset_flag();
        ah.shutdown();

        assert!(matches!(fs::metadata(TEST_DIR), Err(_)))

    }

    struct DummyVm(Mutex<Option<tokio::sync::oneshot::Sender<String>>>);
    impl DummyVm {
        pub fn new(tx: tokio::sync::oneshot::Sender<String>) -> Self {
            DummyVm(Mutex::new(Some(tx)))
        }
    }
    #[async_trait]
    impl GlobalViewModel for DummyVm {
        async fn name_updated(&self, name: String) {
            let mut lock = self.0.lock().await;
            lock.take().unwrap().send(name).unwrap();
        }

        async fn pic_updated(&self, pic: BlobHash) {

        }

        async fn scanning_updated(&self, scanning: bool) {

        }

        async fn nearby_profiles_updated(&self, profiles: Vec<NearbyProfile>) {

        }

        async fn status_updated(&self, status: Status) {

        }

        async fn debug_state_updated(&self, status: DebugState) {

        }

        async fn all_messages_updated(&self, messages: Vec<DisplayMessage>) {

        }

        async fn received_one_message(&self, message: DisplayMessage) {

        }
    }

    #[test]
    fn view_model_works() {
        wipe_test_dir(None);

        fs::create_dir(TEST_DIR).unwrap();
        let ah = AppHost::new(AppConfig::new(TEST_DIR.into()) );
        let _c = ah.rt.enter();

        let (tx, rx) = tokio::sync::oneshot::channel();
        let dvm = Arc::new(DummyVm::new(tx));
        println!("constructing global");
        let g = ah.global(dvm.clone());

        ah.rt.block_on(async {
            g.set_name(String::from("kevin")).await.unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
        });
        let x = rx.blocking_recv().unwrap();

        assert_eq!(x, "kevin")
    }

    #[test]
    fn expiry() {
        let rt = Runtime::new().unwrap();
        let _c = rt.enter();
        let r: Result<(), anyhow::Error> = rt.block_on(async move {
            let node: iroh::node::MemNode = iroh::node::Node::memory().spawn().await.unwrap();
            let doc = node.docs().create().await?;
            // let doc = (doc, node.clone());
            let mut stream = doc.subscribe().await?;
            tokio::spawn(async move {
                println!("entering my loop");
                while let Some(e) = stream.next().await {
                    println!("got an e!")
                };
                println!("DONEEEEEEEEEEE!");
            });
            doc.set_bytes(node.authors().default().await.unwrap(), "boo", "boo").await?;
            tokio::time::sleep(Duration::from_secs(2)).await;

            println!("doing leave");
            doc.leave().await?;
            println!("doing close");
            doc.close().await?;
            println!("doing drop!");
            drop(doc);
            println!("doing shutdown");
            // node.docs().drop_doc();
            node.shutdown().await?;

            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("test done!");
            Ok(())
        });
        r.unwrap();
    }

}
