use crate::util::config::ForceConfig;
use anyhow::Result;
use ckb_sdk::HttpRpcClient;
use force_sdk::indexer::IndexerRpcClient;
use shellexpand::tilde;
use sqlx::MySqlPool;

pub struct CkbIndexer {
    pub force_config: ForceConfig,
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    pub db: MySqlPool,
}

impl CkbIndexer {
    pub async fn new(
        config_path: String,
        db_path: String,
        rpc_url: String,
        indexer_url: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let db = MySqlPool::connect(&db_path).await?;
        Ok(CkbIndexer {
            force_config,
            rpc_client,
            indexer_client,
            db,
        })
    }

    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
}
