use backend::Backend;
use query_builder::*;
use query_source::Column;
use result::QueryResult;

use crate::{associations::HasTable, QuerySource};

/// TODO
pub trait BulkChangesetAssignment<DB: Backend> {
    /// The table these columns belong to
    type Table;

    /// TODO
    fn walk_ast(&self, out: AstPass<DB>) -> QueryResult<()>;
}

impl<C, DB> BulkChangesetAssignment<DB> for C
where
    DB: Backend,
    C: Column,
    C::Table: HasTable,
    <<<C as Column>::Table as HasTable>::Table as QuerySource>::FromClause: QueryFragment<DB>,
{
    type Table = <C as Column>::Table;

    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        QueryFragment::<DB>::walk_ast(&Self::Table::table().from_clause(), out.reborrow())?;
        out.push_sql(".");

        out.push_identifier(C::NAME)?;
        out.push_sql(" = ");
        out.push_identifier("changeset")?;
        out.push_sql(".");
        out.push_identifier(C::NAME)?;
        Ok(())
    }
}
