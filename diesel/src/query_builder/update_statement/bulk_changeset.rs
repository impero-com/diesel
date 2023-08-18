use std::marker::PhantomData;

use crate::backend::Backend;
use crate::pg::Pg;
use crate::prelude::*;
use crate::query_builder::{AstPass, QueryFragment};
use expression::operators::Eq;

/// TODO
pub trait AsBulkChangeset {
    /// TODO
    type ColumnSet;
    /// TODO
    type ColumnInfo;
    /// TODO
    type BulkChangeset;

    /// TODO
    fn as_bulk_changeset(self) -> Self::BulkChangeset;
}

/// TODO
#[derive(Debug, Clone, Copy, Default)]
pub struct AsSet;
/// TODO
#[derive(Debug, Clone, Copy, Default)]
pub struct AsInfo;

/// TODO
#[derive(Debug)]
pub struct BulkChangesetColumn<T, A = AsInfo>(PhantomData<(T, A)>);

impl<T, A> Default for BulkChangesetColumn<T, A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C> QueryFragment<Pg> for BulkChangesetColumn<C, AsSet>
where
    C: Column,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_identifier(C::NAME)?;

        out.push_sql(" = changeset.");
        out.push_identifier("changeset")?;
        out.push_sql(".");

        out.push_identifier(C::NAME)
    }
}

impl<C> QueryFragment<Pg> for BulkChangesetColumn<C, AsInfo>
where
    C: Column,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_identifier(C::NAME)
    }
}

impl<T> AsBulkChangeset for BulkChangesetColumn<T>
where
    T: Column,
{
    type ColumnSet = BulkChangesetColumn<T, AsSet>;
    type ColumnInfo = Self;

    type BulkChangeset = ();

    fn as_bulk_changeset(self) -> Self::BulkChangeset {
        ()
    }
}

// Optional fields are not supported because partial updates must be uniform across the bulk.
// impl<T: AsBulkChangeset> AsBulkChangeset for Option<T> {
//     type ColumnSet = T::ColumnSet;
//     type ColumnInfo = T::ColumnInfo;
//     type BulkChangeset = Option<T::BulkChangeset>;

//     fn as_bulk_changeset(self) -> Self::BulkChangeset {
//         self.map(|v| v.as_bulk_changeset())
//     }
// }

impl<Left, Right> AsBulkChangeset for Eq<Left, Right>
where
    Left: Column,
    Right: AppearsOnTable<Left::Table>,
{
    type ColumnSet = ();
    type ColumnInfo = ();
    type BulkChangeset = BulkChangesetAssign<Left, Right>;

    fn as_bulk_changeset(self) -> Self::BulkChangeset {
        BulkChangesetAssign {
            _column: self.left,
            expr: self.right,
        }
    }
}

/// TODO
#[derive(Debug)]
pub struct BulkChangesetAssign<Col, Expr> {
    _column: Col,
    expr: Expr,
}

impl<DB, Col, Expr> QueryFragment<DB> for BulkChangesetAssign<Col, Expr>
where
    DB: Backend,
    Col: Column,
    Expr: QueryFragment<DB>,
{
    fn walk_ast(&self, out: AstPass<DB>) -> QueryResult<()> {
        QueryFragment::walk_ast(&self.expr, out)
    }
}
