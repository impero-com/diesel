use crate::{
    backend::Backend,
    query_builder::AstPass,
    types::{HasSqlType, ToSql},
    Column, QueryResult,
};

/// A trait used to represent a type which can be bound to a query. Both simple types and composite types.
pub trait Bindable<O, DB: Backend, M> {
    /// The table this type is being bound for. This ensures that all columns are coming from the same source
    type Table;
    /// Bind the given value to the query.
    fn bind(value: &O, pass: AstPass<DB>) -> QueryResult<()>;
}

impl<C, V, DB> Bindable<V, DB, ()> for C
where
    C: Column,
    V: ToSql<C::SqlType, DB>,
    DB: Backend + HasSqlType<C::SqlType>,
{
    type Table = <C as Column>::Table;
    fn bind(v: &V, mut pass: AstPass<DB>) -> QueryResult<()> {
        pass.push_bind_param(v)
    }
}

// impl<C1, C2, V1, V2, Tab> Bindable<(V1, V2), Tuples> for (C1, C2)
// where
//     C1: Bindable<V1, (), Table = Tab>,
//     C2: Bindable<V2, (), Table = Tab>
// {
//     type Table = Tab;
//     fn bind((v1, v2): &(V1, V2), mut pass: AstPass<Pg>) -> QueryResult<()> {
//         C1::bind(v1, pass.reborrow())?;
//         pass.push_sql(", ");
//         C2::bind(v2, pass.reborrow())
//     }
// }
