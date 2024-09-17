mod dsl_impls;

use diesel::{
    expression::ValidGrouping,
    pg::Pg,
    query_builder::{AsQuery, FromClause, QueryFragment, QueryId, SelectStatement},
    Expression, QuerySource, SelectableExpression,
};

pub trait PgSchemaSource {
    type Target;

    fn target(&self) -> &Self::Target;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PgSchema<Source> {
    pub(crate) schema_name: &'static str,
    pub(crate) source: Source,
}

impl<S> PgSchema<S> {
    pub fn new(schema_name: &'static str, source: S) -> Self {
        Self {
            schema_name,
            source,
        }
    }
}

impl<S> QueryId for PgSchema<S>
where
    Self: 'static,
    S: PgSchemaSource,
    S::Target: QueryId,
{
    type QueryId = Self;
}

impl<S> QuerySource for PgSchema<S>
where
    Self: Clone,
    S: PgSchemaSource,
    S::Target: QuerySource,
    <S::Target as QuerySource>::DefaultSelection: SelectableExpression<Self>,
{
    type FromClause = Self;

    type DefaultSelection = <S::Target as QuerySource>::DefaultSelection;

    fn from_clause(&self) -> Self::FromClause {
        self.clone()
    }

    fn default_selection(&self) -> Self::DefaultSelection {
        self.source.target().default_selection()
    }
}

impl<S> QueryFragment<Pg> for PgSchema<S>
where
    S: PgSchemaSource,
    S::Target: QueryFragment<Pg>,
{
    fn walk_ast<'b>(
        &'b self,
        mut pass: diesel::query_builder::AstPass<'_, 'b, Pg>,
    ) -> diesel::QueryResult<()> {
        pass.push_identifier(&self.schema_name)?;
        pass.push_sql(".");
        self.source.target().walk_ast(pass.reborrow())?;
        Ok(())
    }
}

impl<S> AsQuery for PgSchema<S>
where
    S: PgSchemaSource,
    S::Target: AsQuery,
    Self: QuerySource,
    <Self as QuerySource>::DefaultSelection: ValidGrouping<()>,
{
    type SqlType = <<Self as QuerySource>::DefaultSelection as Expression>::SqlType;
    type Query = SelectStatement<FromClause<Self>>;

    fn as_query(self) -> Self::Query {
        SelectStatement::simple(self)
    }
}