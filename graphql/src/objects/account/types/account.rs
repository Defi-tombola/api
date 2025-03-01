use async_graphql::{Context, Object};
use entity::prelude::AccountModel;

pub struct AccountType(AccountModel);

impl From<AccountModel> for AccountType {
    fn from(item: AccountModel) -> Self {
        AccountType(item)
    }
}

#[Object]
impl AccountType {
    async fn name(&self) -> &Option<String> {
        &self.0.name
    }

    async fn address(&self) -> &str {
        &self.0.address
    }

    async fn avatar(&self, ctx: &Context<'_>) -> Option<String> {
        if let Some(ref avatar) = self.0.avatar {
            // let services = ctx.data_unchecked::<ServiceProvider>();
            // let config_service = services.get_service_unchecked::<ConfigService>().await;

            // return Some(format!(
            //     "{bucket_url}/avatars/{avatar}",
            //     bucket_url = config_service.aws.s3.bucket_url,
            // ));
            return None;
        }
        
        // We should return cdn url here

        None
    }

    async fn twitter(&self) -> &Option<String> {
        &self.0.twitter
    }

    async fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
    
    async fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
        }
}
