use diesel::{
    dsl,
    expression::{TypedExpressionType, ValidGrouping},
    expression_methods::EqAll,
    query_builder::{AsQuery, FromClause, Query, SelectStatement},
    query_dsl::methods::{
        BoxedDsl, DistinctOnDsl, FilterDsl, FindDsl, GroupByDsl, LimitDsl, LockingDsl, OffsetDsl,
        OrFilterDsl, OrderDsl, SelectDsl, ThenOrderDsl,
    },
    CombineDsl, Expression, QueryDsl, QuerySource, RunQueryDsl, SelectableExpression, Table,
};

use super::{PgSchema, PgSchemaSource};

impl<S: PgSchemaSource> QueryDsl for PgSchema<S> {}

impl<S, Predicate> FilterDsl<Predicate> for PgSchema<S>
where
    Self: AsQuery,
    <Self as AsQuery>::Query: FilterDsl<Predicate>,
{
    type Output = dsl::Filter<<Self as AsQuery>::Query, Predicate>;

    fn filter(self, predicate: Predicate) -> Self::Output {
        self.as_query().filter(predicate)
    }
}

impl<S, Selection> SelectDsl<Selection> for PgSchema<S>
where
    Selection: Expression,
    Self: AsQuery,
    <Self as AsQuery>::Query: SelectDsl<Selection>,
{
    type Output = dsl::Select<<Self as AsQuery>::Query, Selection>;

    fn select(self, selection: Selection) -> Self::Output {
        self.as_query().select(selection)
    }
}

impl<S, PK> FindDsl<PK> for PgSchema<S>
where
    S: PgSchemaSource,
    S::Target: Table,
    <S::Target as Table>::PrimaryKey: EqAll<PK>,
    Self: FilterDsl<<<S::Target as Table>::PrimaryKey as EqAll<PK>>::Output>,
{
    type Output = dsl::Filter<Self, <<S::Target as Table>::PrimaryKey as EqAll<PK>>::Output>;

    fn find(self, id: PK) -> Self::Output {
        let primary_key = self.source.target().primary_key();
        FilterDsl::filter(self, primary_key.eq_all(id))
    }
}

impl<'a, S, DB> BoxedDsl<'a, DB> for PgSchema<S>
where
    PgSchema<S>: QuerySource + AsQuery<Query = SelectStatement<FromClause<PgSchema<S>>>>,
    SelectStatement<FromClause<PgSchema<S>>>: BoxedDsl<'a, DB>,
    <PgSchema<S> as QuerySource>::DefaultSelection:
        Expression<SqlType = <PgSchema<S> as AsQuery>::SqlType> + ValidGrouping<()>,
    <PgSchema<S> as AsQuery>::SqlType: TypedExpressionType,
{
    type Output = dsl::IntoBoxed<'a, SelectStatement<FromClause<PgSchema<S>>>, DB>;

    fn internal_into_boxed(self) -> Self::Output {
        self.as_query().internal_into_boxed()
    }
}

impl<S> CombineDsl for PgSchema<S>
where
    S: PgSchemaSource,
    S::Target: Table + Copy,
    Self: AsQuery,
{
    type Query = <S::Target as AsQuery>::Query;

    fn union<Rhs>(self, rhs: Rhs) -> dsl::Union<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().union(rhs)
    }

    fn union_all<Rhs>(self, rhs: Rhs) -> dsl::UnionAll<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().union_all(rhs)
    }

    fn intersect<Rhs>(self, rhs: Rhs) -> dsl::Intersect<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().intersect(rhs)
    }

    fn intersect_all<Rhs>(self, rhs: Rhs) -> dsl::IntersectAll<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().intersect_all(rhs)
    }

    fn except<Rhs>(self, rhs: Rhs) -> dsl::Except<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().except(rhs)
    }

    fn except_all<Rhs>(self, rhs: Rhs) -> dsl::ExceptAll<Self, Rhs>
    where
        Rhs: AsQuery<SqlType = <<S::Target as AsQuery>::Query as Query>::SqlType>,
    {
        self.source.target().except_all(rhs)
    }
}

impl<S, Selection> DistinctOnDsl<Selection> for PgSchema<S>
where
    S: PgSchemaSource,
    Selection: SelectableExpression<Self>,
    Self: QuerySource + AsQuery<Query = SelectStatement<FromClause<Self>>>,
    SelectStatement<FromClause<Self>>: DistinctOnDsl<Selection>,
    <Self as QuerySource>::DefaultSelection:
        Expression<SqlType = <Self as AsQuery>::SqlType> + ValidGrouping<()>,
    <Self as AsQuery>::SqlType: TypedExpressionType,
{
    type Output = dsl::DistinctOn<SelectStatement<FromClause<Self>>, Selection>;

    fn distinct_on(self, selection: Selection) -> dsl::DistinctOn<Self, Selection> {
        DistinctOnDsl::distinct_on(self.as_query(), selection)
    }
}

impl<S, Predicate> OrFilterDsl<Predicate> for PgSchema<S>
where
    Self: AsQuery,
    <Self as AsQuery>::Query: OrFilterDsl<Predicate>,
{
    type Output = dsl::OrFilter<<Self as AsQuery>::Query, Predicate>;

    fn or_filter(self, predicate: Predicate) -> Self::Output {
        self.as_query().or_filter(predicate)
    }
}

impl<S, Expr> GroupByDsl<Expr> for PgSchema<S>
where
    Expr: Expression,
    Self: QuerySource + AsQuery<Query = SelectStatement<FromClause<Self>>>,
    <Self as QuerySource>::DefaultSelection:
        Expression<SqlType = <Self as AsQuery>::SqlType> + ValidGrouping<()>,
    <Self as AsQuery>::SqlType: TypedExpressionType,
    <Self as AsQuery>::Query: GroupByDsl<Expr>,
{
    type Output = dsl::GroupBy<SelectStatement<FromClause<Self>>, Expr>;

    fn group_by(self, expr: Expr) -> dsl::GroupBy<Self, Expr> {
        GroupByDsl::group_by(self.as_query(), expr)
    }
}

impl<S> LimitDsl for PgSchema<S>
where
    Self: AsQuery,
    <Self as AsQuery>::Query: LimitDsl,
{
    type Output = <<Self as AsQuery>::Query as LimitDsl>::Output;

    fn limit(self, limit: i64) -> Self::Output {
        self.as_query().limit(limit)
    }
}

impl<S, Lock> LockingDsl<Lock> for PgSchema<S>
where
    Self: QuerySource + AsQuery<Query = SelectStatement<FromClause<Self>>>,
    <Self as QuerySource>::DefaultSelection:
        Expression<SqlType = <Self as AsQuery>::SqlType> + ValidGrouping<()>,
    <Self as AsQuery>::SqlType: TypedExpressionType,
{
    type Output = <SelectStatement<FromClause<Self>> as LockingDsl<Lock>>::Output;

    fn with_lock(self, lock: Lock) -> Self::Output {
        self.as_query().with_lock(lock)
    }
}

impl<S: PgSchemaSource, Conn> RunQueryDsl<Conn> for PgSchema<S> {}

impl<S> OffsetDsl for PgSchema<S>
where
    Self: AsQuery,
    <Self as AsQuery>::Query: OffsetDsl,
{
    type Output = <<Self as AsQuery>::Query as OffsetDsl>::Output;

    fn offset(self, offset: i64) -> Self::Output {
        self.as_query().offset(offset)
    }
}

impl<S, Expr> OrderDsl<Expr> for PgSchema<S>
where
    Expr: Expression,
    Self: AsQuery,
    <Self as AsQuery>::Query: OrderDsl<Expr>,
{
    type Output = <<Self as AsQuery>::Query as OrderDsl<Expr>>::Output;

    fn order(self, expr: Expr) -> Self::Output {
        self.as_query().order(expr)
    }
}

impl<S, Expr> ThenOrderDsl<Expr> for PgSchema<S>
where
    Expr: Expression,
    Self: AsQuery,
    <Self as AsQuery>::Query: ThenOrderDsl<Expr>,
{
    type Output = <<Self as AsQuery>::Query as ThenOrderDsl<Expr>>::Output;

    fn then_order_by(self, expr: Expr) -> Self::Output {
        self.as_query().then_order_by(expr)
    }
}
