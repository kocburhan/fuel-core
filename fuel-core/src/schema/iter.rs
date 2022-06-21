use crate::database::{Database, KvStoreError};
use crate::schema::scalars::{AssetId, ContractId, HexString, Salt, U64};
use crate::state::IterDirection;
use anyhow::anyhow;
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Context, InputObject, Object,
};
use fuel_core_interfaces::common::{fuel_storage::Storage, fuel_tx, fuel_types, fuel_vm};

async fn paginate<
    I: std::marker::Sync + std::marker::Send + async_graphql::connection::CursorType,
    O: async_graphql::OutputType,
>(
    iter: impl Iterator,
) -> async_graphql::Result<Connection<I, O, EmptyFields, EmptyFields>> {
    query(
        after,
        before,
        first,
        last,
        |after: Option<I>, before: Option<I>, first, last| async move {
            // Calculate direction of which to iterate through rocksdb
            let (records_to_fetch, direction) = if let Some(first) = first {
                (first, IterDirection::Forward)
            } else if let Some(last) = last {
                (last, IterDirection::Reverse)
            } else {
                (0, IterDirection::Forward)
            };

            if (first.is_some() && before.is_some())
                || (after.is_some() && before.is_some())
                || (last.is_some() && after.is_some())
            {
                return Err(anyhow!("Wrong argument combination"));
            }

            let after = after.map(I::from);
            let before = before.map(I::from);

            let start = if direction == IterDirection::Forward {
                after
            } else {
                before
            };

            let mut iter = db.contract_balances(filter.contract.into(), start, Some(direction));

            let mut started = None;
            if start.is_some() {
                started = iter.next();
            }

            let mut return_values = iter
                .take(records_to_fetch + 1)
                .map(|balance| {
                    let balance = balance?;

                    Ok(O)
                })
                .collect::<Result<Vec<O>, KvStoreError>>()?;

            let has_next_page = return_values.len() > records_to_fetch;

            if has_next_page {
                return_values.pop();
            }

            if direction == IterDirection::Reverse {
                return_values.reverse();
            }

            let mut connection = Connection::new(started.is_some(), has_next_page);
            connection.edges.extend(
                return_values
                    .into_iter()
                    .map(|item| Edge::new(item.asset_id.into(), item)),
            );

            Ok::<Connection<I, O>, anyhow::Error>(connection)
        },
    )
    .await
}
