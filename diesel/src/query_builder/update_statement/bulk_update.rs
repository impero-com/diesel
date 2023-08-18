use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::associations::HasTable;
use crate::pg::Pg;
use crate::prelude::*;
use crate::query_builder::{AsChangeset, ColumnList};
use crate::query_builder::{AstPass, QueryFragment, QueryId};

use super::bindable::Bindable;
use super::bulk_changeset::AsBulkChangeset;
use super::BulkChangesetAssignment;

/// TODO
#[derive(Debug)]
pub struct BulkUpdate<I, T, Marker>
where
    T: AsChangeset + AsBulkChangeset,
    T::Target: Table,
    <T::Target as Table>::PrimaryKey: Expression,
    I: Ord,
{
    changes: BTreeMap<I, T::BulkChangeset>,
    _marker: PhantomData<Marker>,
}

impl<I, T, M> BulkUpdate<I, T, M>
where
    T: AsChangeset + AsBulkChangeset,
    T::Target: Table,
    <T::Target as Table>::PrimaryKey: Expression,
    I: Ord,
{
    /// TODO
    pub fn new(changes: BTreeMap<I, T>) -> Self {
        Self {
            changes: changes
                .into_iter()
                .map(|(i, c)| (i, c.as_bulk_changeset()))
                .collect(),
            _marker: PhantomData,
        }
    }
}

impl<T, I, Marker> QueryFragment<Pg> for BulkUpdate<I, T, Marker>
where
    I: Ord,
    T: AsChangeset + AsBulkChangeset,
    T::Target: Table + HasTable<Table = T::Target>,

    T::Changeset: QueryFragment<Pg>,
    T::BulkChangeset: QueryFragment<Pg>,
    T::ColumnInfo: Default + QueryFragment<Pg>,
    T::ColumnSet: Default + QueryFragment<Pg>,

    <T::Target as QuerySource>::FromClause: QueryFragment<Pg>,
    <T::Target as Table>::PrimaryKey: Default + QueryFragment<Pg>,
    <T::Target as Table>::PrimaryKey:
        Bindable<I, Pg, Marker> + ColumnList + BulkChangesetAssignment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        out.push_sql("UPDATE ");
        T::Target::table().from_clause().walk_ast(out.reborrow())?;
        out.push_sql(" SET ");
        T::ColumnSet::default().walk_ast(out.reborrow())?;
        out.push_sql(" FROM (VALUES ");
        for (idx, (id, change)) in self.changes.iter().enumerate() {
            if idx > 0 {
                out.push_sql(", ");
            }
            out.push_sql("(");
            <T::Target as Table>::PrimaryKey::bind(id, out.reborrow())?;
            out.push_sql(", ");
            // TODO: this skips options
            change.walk_ast(out.reborrow())?;
            out.push_sql(")");
        }
        out.push_sql(") AS ");
        out.push_identifier("changeset")?;
        out.push_sql(" (");

        ColumnList::walk_ast(
            &Table::primary_key(&<T::Target as HasTable>::table()),
            out.reborrow(),
        )?;

        out.push_sql(", ");
        T::ColumnInfo::default().walk_ast(out.reborrow())?;
        out.push_sql(") WHERE ");

        BulkChangesetAssignment::walk_ast(
            &Table::primary_key(&<T::Target as HasTable>::table()),
            out.reborrow(),
        )?;

        Ok(())
    }
}

impl<I, T, M> QueryId for BulkUpdate<I, T, M>
where
    T::Target: Table,
    T: AsChangeset + AsBulkChangeset,
    I: Ord,
{
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}
impl<I, T, Conn, M> RunQueryDsl<Conn> for BulkUpdate<I, T, M>
where
    T::Target: Table,
    T: AsChangeset + AsBulkChangeset,
    I: Ord,
{
}
