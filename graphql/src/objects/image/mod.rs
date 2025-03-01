// mod inputs;

// use self::inputs::{EntityType, ImageUploadInput};
// use crate::guards::auth::AuthGuard;
// use crate::objects::GQLJWTData;
// use async_graphql::{Context, Object, Upload};
// use chrono::Utc;
// use service::account::store::AccountStore;
// use service::account::AccountService;
// use service::{
//     account::types::UpdateAccount,
//     prelude::{ConfigService, StoreService},
//     services::ServiceProvider,
// };
// use tracing::warn;
// use uuid::Uuid;

// #[derive(Default)]
// pub struct ImageMutation;

// #[Object]
// impl ImageMutation {
//     #[graphql(guard = "AuthGuard::new()")]
//     async fn request_image_upload(
//         &self,
//         ctx: &Context<'_>,
//         input: ImageUploadInput,
//     ) -> async_graphql::Result<String, async_graphql::Error> {
//         let services = ctx.data_unchecked::<ServiceProvider>();

//         let store_service = services.get_service_unchecked::<StoreService>().await;
//         let aws_s3_service = services.get_service_unchecked::<AWSS3Service>().await;
//         let account_service = services.get_service_unchecked::<AccountService>().await;

//         let claims = ctx
//             .data_opt::<GQLJWTData>()
//             .and_then(|rd| rd.claims.as_ref())
//             .unwrap();

//         match input.entity_type {
//             EntityType::Vault => {
//                 let vault_service = services.get_service_unchecked::<VaultService>().await;

//                 let pool = store_service.read();
//                 let vault = VaultStore::try_find_by_uid(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault not found"))?;

//                 let manager = AccountStore::try_find_by_vault_id(pool, vault.id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault manager not found"))?;

//                 // Verify if user can manage provided vault
//                 if !manager.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this vault",
//                     ));
//                 }

//                 let filename = vault
//                     .avatar
//                     .clone()
//                     .and_then(|i| if is_default_avatar(&i) { None } else { Some(i) })
//                     .unwrap_or(Uuid::new_v4().to_string());

//                 let url = aws_s3_service
//                     .generate_upload_url(filename.clone(), "avatars".to_string())
//                     .await
//                     .map_err(|e| {
//                         warn!("{:?}", e);
//                         async_graphql::Error::new("Failed to sign url")
//                     })?;

//                 if vault.avatar.is_none() || is_default_avatar(vault.avatar.as_ref().unwrap()) {
//                     let dto = UpdateVault::builder()
//                         .and_avatar(Some(filename))
//                         .updated_at(Utc::now())
//                         .build();

//                     let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                         warn!("Failed to start transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                     vault_service
//                         .update(vault, dto, None, &mut db_tx)
//                         .await
//                         .map_err(|_| async_graphql::Error::new("Failed to update vault"))?;

//                     store_service.commit_transaction(db_tx).await.map_err(|e| {
//                         warn!("Failed to commit transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;
//                 }

//                 Ok(url)
//             }
//             EntityType::Cover => {
//                 let vault_service = services.get_service_unchecked::<VaultService>().await;

//                 let pool = store_service.read();
//                 let vault = VaultStore::try_find_by_uid(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault not found"))?;

//                 let manager = AccountStore::try_find_by_vault_id(pool, vault.id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault manager not found"))?;

//                 // Verify if user can manage provided vault
//                 if !manager.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this vault",
//                     ));
//                 }

//                 let filename = vault
//                     .cover
//                     .clone()
//                     .and_then(|i| if is_default_cover(&i) { None } else { Some(i) })
//                     .unwrap_or(Uuid::new_v4().to_string());

//                 let url = aws_s3_service
//                     .generate_upload_url(filename.clone(), "covers".to_string())
//                     .await
//                     .map_err(|e| {
//                         warn!("{:?}", e);
//                         async_graphql::Error::new("Failed to sign url")
//                     })?;

//                 if vault.cover.is_none() || is_default_cover(vault.cover.as_ref().unwrap()) {
//                     let dto = UpdateVault::builder()
//                         .and_cover(Some(filename))
//                         .updated_at(Utc::now())
//                         .build();

//                     let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                         warn!("Failed to start transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                     vault_service
//                         .update(vault, dto, None, &mut db_tx)
//                         .await
//                         .map_err(|_| async_graphql::Error::new("Failed to update vault"))?;

//                     store_service.commit_transaction(db_tx).await.map_err(|e| {
//                         warn!("Failed to commit transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;
//                 }

//                 Ok(url)
//             }
//             EntityType::Account => {
//                 let pool = store_service.read();
//                 let account = AccountStore::try_find_by_address(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Account not found"))?;

//                 // Verify if user can manage provided account
//                 if !account.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this account",
//                     ));
//                 }

//                 let filename = account
//                     .avatar
//                     .clone()
//                     .and_then(|i| if is_default_avatar(&i) { None } else { Some(i) })
//                     .unwrap_or(Uuid::new_v4().to_string());

//                 let url = aws_s3_service
//                     .generate_upload_url(filename.clone(), "avatars".to_string())
//                     .await
//                     .map_err(|e| {
//                         warn!("{:?}", e);
//                         async_graphql::Error::new("Failed to sign url")
//                     })?;

//                 if account.avatar.is_none() || is_default_avatar(account.avatar.as_ref().unwrap()) {
//                     let dto = UpdateAccount {
//                         avatar: Some(filename),
//                         updated_at: Utc::now(),
//                         ..Default::default()
//                     };

//                     let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                         warn!("Failed to start transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                     account_service
//                         .update(account.id, dto, &mut db_tx)
//                         .await
//                         .map_err(|_| async_graphql::Error::new("Failed to update account"))?;

//                     store_service.commit_transaction(db_tx).await.map_err(|e| {
//                         warn!("Failed to commit transaction: {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;
//                 }

//                 Ok(url)
//             }
//         }
//     }

//     #[graphql(guard = "AuthGuard::new()")]
//     async fn upload_image(
//         &self,
//         ctx: &Context<'_>,
//         input: ImageUploadInput,
//         file: Upload,
//     ) -> async_graphql::Result<String, async_graphql::Error> {
//         let services = ctx.data_unchecked::<ServiceProvider>();

//         let config_service = services.get_service_unchecked::<ConfigService>().await;
//         let store_service = services.get_service_unchecked::<StoreService>().await;
//         let account_service = services.get_service_unchecked::<AccountService>().await;

//         let claims = ctx
//             .data_opt::<GQLJWTData>()
//             .and_then(|rd| rd.claims.as_ref())
//             .unwrap();

//         let file_value = file.value(ctx).unwrap();
//         let aws_s3_service = services.get_service_unchecked::<AWSS3Service>().await;

//         match input.entity_type {
//             EntityType::Vault => {
//                 let vault_service = services.get_service_unchecked::<VaultService>().await;

//                 let pool = store_service.read();
//                 let vault = VaultStore::try_find_by_uid(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault not found"))?;

//                 let manager = AccountStore::try_find_by_vault_id(pool, vault.id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault manager not found"))?;

//                 // Verify if user can manage provided vault
//                 if !manager.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this vault",
//                     ));
//                 }

//                 let filename = Uuid::new_v4().to_string();
//                 let path = format!("avatars/{filename}");

//                 aws_s3_service
//                     .upload(file_value, path.clone())
//                     .await
//                     .map_err(|e| {
//                         warn!("Failed to upload image {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                 let dto = UpdateVault::builder()
//                     .and_avatar(Some(filename))
//                     .updated_at(Utc::now())
//                     .build();

//                 let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                     warn!("Failed to start transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 vault_service
//                     .update(vault.clone(), dto, None, &mut db_tx)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Failed to update vault"))?;

//                 store_service.commit_transaction(db_tx).await.map_err(|e| {
//                     warn!("Failed to commit transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 // Remove old avatar
//                 if let Some(avatar) = vault.avatar {
//                     if !is_default_avatar(&avatar) {
//                         if let Err(e) = aws_s3_service.remove(format!("avatars/{avatar}")).await {
//                             warn!("Failed to remove image {e:?}");
//                         }
//                     }
//                 }

//                 Ok(format!(
//                     "{cdn}/{path}",
//                     cdn = config_service.aws.s3.bucket_url
//                 ))
//             }
//             EntityType::Cover => {
//                 let service = services.get_service_unchecked::<VaultService>().await;

//                 let pool = store_service.read();
//                 let vault = VaultStore::try_find_by_uid(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault not found"))?;

//                 let manager = AccountStore::try_find_by_vault_id(pool, vault.id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Vault manager not found"))?;

//                 // Verify if user can manage provided vault
//                 if !manager.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this vault",
//                     ));
//                 }

//                 let filename = Uuid::new_v4().to_string();
//                 let path = format!("covers/{filename}");

//                 aws_s3_service
//                     .upload(file_value, path.clone())
//                     .await
//                     .map_err(|e| {
//                         warn!("Failed to upload image {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                 let dto = UpdateVault::builder()
//                     .and_cover(Some(filename))
//                     .updated_at(Utc::now())
//                     .build();

//                 let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                     warn!("Failed to start transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 service
//                     .update(vault.clone(), dto, None, &mut db_tx)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Failed to update vault cover"))?;

//                 store_service.commit_transaction(db_tx).await.map_err(|e| {
//                     warn!("Failed to commit transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 // Remove old cover
//                 if let Some(cover) = vault.cover {
//                     if !is_default_cover(&cover) {
//                         if let Err(e) = aws_s3_service.remove(format!("covers/{cover}")).await {
//                             warn!("Failed to remove image {e:?}");
//                         }
//                     }
//                 }

//                 Ok(format!(
//                     "{cdn}/{path}",
//                     cdn = config_service.aws.s3.bucket_url
//                 ))
//             }
//             EntityType::Account => {
//                 let pool = store_service.read();
//                 let account = AccountStore::try_find_by_address(pool, input.entity_id)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Internal error"))?
//                     .ok_or(async_graphql::Error::new("Account not found"))?;

//                 // Verify if user can manage provided account
//                 if !account.address.eq(&claims.sub) {
//                     return Err(async_graphql::Error::new(
//                         "You are not allowed to change this account",
//                     ));
//                 }

//                 let filename = Uuid::new_v4().to_string();
//                 let path = format!("avatars/{filename}");

//                 aws_s3_service
//                     .upload(file_value, path.clone())
//                     .await
//                     .map_err(|e| {
//                         warn!("Failed to upload image {e:?}");
//                         async_graphql::Error::new("Internal error")
//                     })?;

//                 let dto = UpdateAccount {
//                     avatar: Some(filename),
//                     updated_at: Utc::now(),
//                     ..Default::default()
//                 };

//                 let mut db_tx = store_service.begin_transaction().await.map_err(|e| {
//                     warn!("Failed to start transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 account_service
//                     .update(account.clone().id, dto, &mut db_tx)
//                     .await
//                     .map_err(|_| async_graphql::Error::new("Failed to update account"))?;

//                 store_service.commit_transaction(db_tx).await.map_err(|e| {
//                     warn!("Failed to commit transaction: {e:?}");
//                     async_graphql::Error::new("Internal error")
//                 })?;

//                 // Remove old avatar
//                 if let Some(avatar) = account.avatar {
//                     if !is_default_avatar(&avatar) {
//                         if let Err(e) = aws_s3_service.remove(format!("avatars/{avatar}")).await {
//                             warn!("Failed to remove image {e:?}");
//                         }
//                     }
//                 }

//                 Ok(format!(
//                     "{cdn}/{path}",
//                     cdn = config_service.aws.s3.bucket_url
//                 ))
//             }
//         }
//     }
// }
