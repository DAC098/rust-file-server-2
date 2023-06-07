use std::fmt::Write;

use axum::http::StatusCode;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use lib::ids;
use lib::models::storage::{StorageItem, StorageType};
use lib::models::actions::storage::{UpdateStorage, UpdateStorageType};

use crate::net;
use crate::net::error;
use crate::state::ArcShared;
use crate::auth::initiator;
use crate::util::PgParams;
use crate::storage;
use crate::tags;

pub mod fs;

#[derive(Deserialize)]
pub struct PathParams {
    storage_id: ids::StorageId,
}

pub async fn get(
    State(state): State<ArcShared>,
    initiator: initiator::Initiator,
    Path(PathParams { storage_id }): Path<PathParams>,
) -> error::Result<impl IntoResponse> {
    let conn = state.pool().get().await?;

    let Some(medium) = storage::Medium::retrieve(
        &conn,
        initiator.user().id(),
        &storage_id
    ).await? else {
        return Err(error::Error::new()
            .status(StatusCode::NOT_FOUND)
            .kind("StorageNotFound")
            .message("requested storage item was not found"));
    };

    if medium.deleted.is_some() {
        return Err(error::Error::new()
            .status(StatusCode::NOT_FOUND)
            .kind("StorageNotFound")
            .message("requested storage item was not found"));
    }

    let type_ = match medium.type_ {
        storage::Type::Local(local) => StorageType::Local {
            path: local.path
        }
    };

    let rtn = net::JsonWrapper::new(StorageItem {
        id: medium.id,
        name: medium.name,
        user_id: medium.user_id,
        type_,
        tags: medium.tags,
        created: medium.created,
        updated: medium.updated,
        deleted: medium.deleted
    });

    Ok(net::Json::new(rtn))
}

pub async fn put(
    State(state): State<ArcShared>,
    initiator: initiator::Initiator,
    Path(PathParams { storage_id }): Path<PathParams>,
    axum::Json(json): axum::Json<UpdateStorage>,
) -> error::Result<impl IntoResponse> {
    let mut conn = state.pool().get().await?;

    let Some(medium) = storage::Medium::retrieve(
        &conn,
        initiator.user().id(),
        &storage_id
    ).await? else {
        return Err(error::Error::new()
            .status(StatusCode::NOT_FOUND)
            .kind("StorageNotFound")
            .message("requested storage item was not found"));
    };

    if medium.deleted.is_some() {
        return Err(error::Error::new()
            .status(StatusCode::NOT_FOUND)
            .kind("StorageNotFound")
            .message("requested storage item was not found"));
    }

    if !json.has_work() {
        return Err(error::Error::new()
            .status(StatusCode::BAD_REQUEST)
            .kind("NoWork")
            .message("requested update with no changes"));
    }

    let transaction = conn.transaction().await?;

    if json.name.is_some() || json.type_.is_some() {
        let updated = chrono::Utc::now();
        let mut update_query = String::from("update storage set updated = $2");
        let mut update_params = PgParams::with_capacity(2);
        update_params.push(&storage_id);
        update_params.push(&updated);

        if let Some(name) = &json.name {
            if let Some(found_id) = storage::name_check(&transaction, initiator.user().id(), name).await? {
                if found_id != storage_id {
                    return Err(error::Error::new()
                        .status(StatusCode::BAD_REQUEST)
                        .kind("StorageNameExists")
                        .message("requested storage name already exists"));
                }
            }

            write!(&mut update_query, "name = ${} ", update_params.push(name)).unwrap();
        }

        if let Some(type_) = &json.type_ {
            match type_ {
                UpdateStorageType::Local {..} => {}
            }
        }

        write!(&mut update_query, "where storage_id = $1").unwrap();

        transaction.execute(update_query.as_str(), update_params.as_slice()).await?;
    }

    if let Some(tags) = json.tags {
        tags::update_tags(
            &transaction,
            "storage_tags",
            "storage_id",
            &storage_id,
            &tags
        ).await?;
    }

    transaction.commit().await?;

    Ok(net::Json::empty())
}

pub async fn delete(
    State(state): State<ArcShared>,
    initiator: initiator::Initiator,
    Path(PathParams { storage_id }): Path<PathParams>,
) -> error::Result<impl IntoResponse> {
    let mut conn = state.pool().get().await?;

    let Some(medium) = storage::Medium::retrieve(
        &conn,
        initiator.user().id(),
        &storage_id
    ).await? else {
        return Err(error::Error::new()
            .status(StatusCode::NOT_FOUND)
            .kind("StorageNotFound")
            .message("requested storage item was not found"));
    };

    let deleted = chrono::Utc::now();

    let transaction = conn.transaction().await?;

    // soft delete fs items
    let _ = transaction.execute(
        "update fs set deleted = $2 where storage_id = $1",
        &[&storage_id, &deleted]
    ).await?;

    // soft delete storage item
    let _ = transaction.execute(
        "update storage set deleted = $2 where storage_id = $1",
        &[&storage_id, &deleted]
    ).await?;

    Ok(net::Json::empty()
       .with_message("deleted storage"))
}