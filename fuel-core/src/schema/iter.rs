use crate::database::KvStoreError;
use crate::state::IterDirection;
use anyhow::{anyhow, Error};
use async_graphql::connection::{query, Connection, CursorType, Edge, EmptyFields};
use std::iter::IntoIterator;
use std::marker::{Send, Sync};

async fn paginate<
    I: async_graphql::connection::CursorType + std::marker::Sync + std::marker::Send,
    O: async_graphql::connection::CursorType + std::marker::Sync + std::marker::Send,
    F: FnMut(I) -> O,
>(
    iter: &mut dyn Iterator<Item = I>,
    closure: F,
    first: Option<i32>,
    after: Option<String>,
    last: Option<i32>,
    before: Option<String>,
) where
    <I as CursorType>::Error: std::marker::Send,
    <I as CursorType>::Error: Sync + 'static,
    O: async_graphql::OutputType,
{
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

            let mut started = None;

            if start.is_some() {
                started = iter.next();
            }

            let mut return_values = iter
                .take(records_to_fetch + 1)
                .map(closure)
                .collect::<Vec<O>>();

            let has_next_page = return_values.len() > records_to_fetch;

            if has_next_page {
                return_values.pop();
            }

            if direction == IterDirection::Reverse {
                return_values.reverse();
            }
            let mut connection = Connection::new(started.is_some(), has_next_page);

            connection
                .edges
                .extend(return_values.into_iter().map(|item| Edge::new(item, item)));

            Ok::<Connection<I, O>, anyhow::Error>(connection)
        },
    )
    .await;
}

/*
 * So this is the full function, but breaks in hundreds of places, so just going to write it a bit
 * at a time
 *
async fn paginate<
    I: std::marker::Sync + std::marker::Send + async_graphql::connection::CursorType,
    O: async_graphql::OutputType,
>(
    iter: impl Iterator<Item = Result<(AssetId, u64), Error>>,
    closure: F,
    first: Option<i32>,
    after: Option<String>,
    last: Option<i32>,
    before: Option<String>,
) -> async_graphql::Result<Connection<I, O, EmptyFields, EmptyFields>>
where
    <I as CursorType>::Error: std::marker::Send,
    F: Fn(<impl Iterator as Iterator>::Item) -> O,
{
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

            let mut started = None;
            if start.is_some() {
                started = iter.next();
            }

            let mut return_values = iter
                .take(records_to_fetch + 1)
                .map(closure)
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
*/
