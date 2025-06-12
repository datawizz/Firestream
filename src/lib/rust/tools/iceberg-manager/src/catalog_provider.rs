use datafusion::execution::context::SessionContext;
use iceberg::Catalog;
use iceberg_datafusion::IcebergCatalogProvider;
use std::sync::Arc;

/// Register an Iceberg catalog with a DataFusion context
pub async fn register_iceberg_catalog(
    ctx: &SessionContext,
    catalog_name: &str,
    iceberg_catalog: Arc<dyn Catalog>,
) -> crate::error::Result<()> {
    let provider = Arc::new(
        IcebergCatalogProvider::try_new(iceberg_catalog).await?
    );
    
    ctx.register_catalog(catalog_name, provider);
    
    Ok(())
}
