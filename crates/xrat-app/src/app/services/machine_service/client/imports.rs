use super::*;
use crate::app::import;
use crate::app::services::subscriptions::{self, SubscriptionUpdateSummary};
use std::collections::BTreeMap;
use xrat_db::ImportSummary;

fn body(bytes: &[u8]) -> Result<String> {
    if crate::app::services::xray_profiles::parse(bytes)?.is_some() {
        return Err(AppError::InvalidArgument("Complete Xray profiles require local administrator provisioning; machine catalog imports accept proxy links only".into()));
    }
    import::parse_nodes(bytes)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        AppError::InvalidArgument("subscription contents must be UTF-8 text or base64".into())
    })
}

impl MachineClient {
    pub(crate) async fn import_subscription(
        &self,
        input: &str,
        name: Option<&str>,
        headers: &BTreeMap<String, String>,
    ) -> Result<ImportSummary> {
        self.import_subscription_with_fallback(input, name, None, headers)
            .await
    }

    pub(crate) async fn import_subscription_with_fallback(
        &self,
        input: &str,
        name: Option<&str>,
        fallback_name: Option<&str>,
        headers: &BTreeMap<String, String>,
    ) -> Result<ImportSummary> {
        let (mut source, bytes) = import::load_subscription_input(input, headers).await?;
        source.name = source.name.or_else(|| fallback_name.map(str::to_owned));
        self.import_result(CatalogRequest::Import {
            contents: vec![body(&bytes)?],
            source,
            name: name.map(str::to_owned),
            headers: headers.clone(),
        })
        .await
    }

    pub(crate) async fn import_url_list(
        &self,
        path: &str,
        name: Option<&str>,
    ) -> Result<ImportSummary> {
        let (source, contents) = import::load_url_list_parts(path, body).await?;
        self.import_result(CatalogRequest::Import {
            contents,
            source,
            name: name.map(str::to_owned),
            headers: BTreeMap::new(),
        })
        .await
    }

    async fn import_result(&self, request: CatalogRequest) -> Result<ImportSummary> {
        match self.catalog(request).await? {
            CatalogPayload::Import(summary) => Ok(summary),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn subscription(&self, reference: &str) -> Result<SubscriptionRecord> {
        match self
            .catalog(CatalogRequest::Subscription {
                reference: reference.into(),
            })
            .await?
        {
            CatalogPayload::Subscription(record) => Ok(*record),
            _ => Err(unexpected()),
        }
    }

    pub(crate) async fn refresh_subscriptions(
        &self,
        references: &[String],
    ) -> Result<SubscriptionUpdateSummary> {
        let targets = if references.is_empty() {
            self.subscriptions().await?
        } else {
            let mut targets = Vec::with_capacity(references.len());
            for reference in references {
                targets.push(self.subscription(reference).await?);
            }
            targets
        };
        Ok(
            subscriptions::refresh_targets(targets, |url, headers| async move {
                self.import_subscription(&url, None, &headers).await
            })
            .await,
        )
    }
}
