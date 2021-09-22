use crate::{expr::SimpleExpr, types::LogicalChainOper};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionType {
    Any,
    All,
}

/// Represents the value of an [`Condition::any`] or [`Condition::all`]: a set of disjunctive or conjunctive conditions.
#[derive(Debug, Clone)]
pub struct Condition<'a, DB> {
    pub(crate) condition_type: ConditionType,
    pub(crate) conditions: Vec<ConditionExpression<'a, DB>>,
}

pub trait IntoCondition<'a, DB> {
    fn into_condition(self) -> Condition<'a, DB>;
}

pub type Cond<'a, DB> = Condition<'a, DB>;

/// Represents anything that can be passed to an [`Condition::any`] or [`Condition::all`]'s [`Condition::add`] method.
///
/// The arguments are automatically converted to the right enum.
#[derive(Debug, Clone)]
pub enum ConditionExpression<'a, DB> {
    Condition(Condition<'a, DB>),
    SimpleExpr(SimpleExpr<'a, DB>),
}

#[derive(Debug, Clone)]
pub enum ConditionHolderContents<'a, DB> {
    Empty,
    Chain(Vec<LogicalChainOper<'a, DB>>),
    Condition(Condition<'a, DB>),
}

#[derive(Debug, Clone)]
pub struct ConditionHolder<'a, DB> {
    pub contents: ConditionHolderContents<'a, DB>,
}

impl<'a, DB> Condition<'a, DB> {
    /// Add a condition to the set.
    ///
    /// If it's an [`Condition::any`], it will be separated from the others by an `" OR "` in the query. If it's
    /// an [`Condition::all`], it will be separated by an `" AND "`.
    #[allow(clippy::should_implement_trait)]
    pub fn add<C>(mut self, condition: C) -> Self
    where
        C: Into<ConditionExpression<'a, DB>>,
    {
        let mut expr: ConditionExpression<'a, DB> = condition.into();
        if let ConditionExpression::Condition(ref mut c) = expr {
            // Don't add empty `Condition::any` and `Condition::all`.
            if c.conditions.is_empty() {
                return self;
            }
            // Skip the junction if there is only one.
            if c.conditions.len() == 1 {
                expr = c.conditions.pop().unwrap();
            }
        }
        self.conditions.push(expr);
        self
    }

    /// Add an optional condition to the set.
    ///
    /// Shorthand for `if o.is_some() { self.add(o) }`
    #[allow(clippy::should_implement_trait)]
    pub fn add_option<C>(self, other: Option<C>) -> Self
    where
        C: Into<ConditionExpression<'a, DB>>,
    {
        if let Some(other) = other {
            self.add(other)
        } else {
            self
        }
    }

    /// Create a condition that is true if any of the conditions is true.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .cond_where(
    ///         Cond::any()
    ///             .add(Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]))
    ///             .add(Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%"))
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `image` FROM `glyph` WHERE `glyph`.`aspect` IN (3, 4) OR `glyph`.`image` LIKE 'A%'"#
    /// );
    /// ```
    pub fn any() -> Condition<'a, DB> {
        Condition {
            condition_type: ConditionType::Any,
            conditions: Vec::new(),
        }
    }

    /// Create a condition that is false if any of the conditions is false.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .cond_where(
    ///         Cond::all()
    ///             .add(Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]))
    ///             .add(Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%"))
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `image` FROM `glyph` WHERE `glyph`.`aspect` IN (3, 4) AND `glyph`.`image` LIKE 'A%'"#
    /// );
    /// ```
    pub fn all() -> Condition<'a, DB> {
        Condition {
            condition_type: ConditionType::All,
            conditions: Vec::new(),
        }
    }
}

impl<'a, DB> std::convert::From<Condition<'a, DB>> for ConditionExpression<'a, DB> {
    fn from(condition: Condition<'a, DB>) -> Self {
        ConditionExpression::Condition(condition)
    }
}

impl<'a, DB> std::convert::From<SimpleExpr<'a, DB>> for ConditionExpression<'a, DB> {
    fn from(condition: SimpleExpr<'a, DB>) -> Self {
        ConditionExpression::SimpleExpr(condition)
    }
}

/// Macro to easily create an [`Condition::any`].
///
/// # Examples
///
/// ```
/// use sea_query::{*, tests_cfg::*};
///
/// let query = Query::select()
///     .column(Glyph::Image)
///     .from(Glyph::Table)
///     .cond_where(
///         any![
///             Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]),
///             Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%")
///         ]
///     )
///     .to_owned();
///
/// assert_eq!(
///     query.to_string(MysqlQueryBuilder),
///     r#"SELECT `image` FROM `glyph` WHERE `glyph`.`aspect` IN (3, 4) OR `glyph`.`image` LIKE 'A%'"#
/// );
/// ```
#[macro_export]
macro_rules! any {
    ( $( $x:expr ),* ) => {
        {
            let mut tmp = sea_query::Condition::any();
            $(
                tmp = tmp.add($x);
            )*
            tmp
        }
    };
}

/// Macro to easily create an [`Condition::all`].
///
/// # Examples
///
/// ```
/// use sea_query::{*, tests_cfg::*};
///
/// let query = Query::select()
///     .column(Glyph::Image)
///     .from(Glyph::Table)
///     .cond_where(
///         all![
///             Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]),
///             Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%")
///         ]
///     )
///     .to_owned();
///
/// assert_eq!(
///     query.to_string(MysqlQueryBuilder),
///     r#"SELECT `image` FROM `glyph` WHERE `glyph`.`aspect` IN (3, 4) AND `glyph`.`image` LIKE 'A%'"#
/// );
#[macro_export]
macro_rules! all {
    ( $( $x:expr ),* ) => {
        {
            let mut tmp = sea_query::Condition::all();
            $(
                tmp = tmp.add($x);
            )*
            tmp
        }
    };
}

pub trait ConditionalStatement<'a, DB> {
    /// And where condition. This cannot be mixed with [`ConditionalStatement::or_where`].
    /// Calling `or_where` after `and_where` will panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .and_where(Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]))
    ///     .and_where(Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `image` FROM `glyph` WHERE `glyph`.`aspect` IN (3, 4) AND `glyph`.`image` LIKE 'A%'"#
    /// );
    /// ```
    fn and_where(&mut self, other: SimpleExpr<'a, DB>) -> &mut Self {
        self.cond_where(other)
    }

    /// Optional and where, short hand for `if c.is_some() q.and_where(c)`.
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .and_where(Expr::col(Glyph::Aspect).is_in(vec![3, 4]))
    ///     .and_where_option(Some(Expr::col(Glyph::Image).like(&"A%")))
    ///     .and_where_option(None)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `image` FROM `glyph` WHERE `aspect` IN (3, 4) AND `image` LIKE 'A%'"#
    /// );
    /// ```
    fn and_where_option(&mut self, other: Option<SimpleExpr<'a, DB>>) -> &mut Self {
        if let Some(other) = other {
            self.and_where(other);
        }
        self
    }

    #[deprecated(
        since = "0.12.0",
        note = "Please use [`ConditionalStatement::cond_where`]. Calling `or_where` after `and_where` will panic."
    )]
    /// Or where condition. This cannot be mixed with [`ConditionalStatement::and_where`].
    /// Calling `or_where` after `and_where` will panic.
    fn or_where(&mut self, other: SimpleExpr<'a, DB>) -> &mut Self {
        self.and_or_where(LogicalChainOper::Or(other))
    }

    #[doc(hidden)]
    // Trait implementation.
    fn and_or_where(&mut self, condition: LogicalChainOper<'a, DB>) -> &mut Self;

    /// Where condition, expressed with `any` and `all`.
    /// Calling `cond_where` multiple times will conjoin them.
    /// Calling `or_where` after `cond_where` will panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .cond_where(
    ///         Cond::all()
    ///             .add(Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]))
    ///             .add(Cond::any()
    ///                 .add(Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%"))
    ///                 .add(Expr::tbl(Glyph::Table, Glyph::Image).like(&"B%"))
    ///             )
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(),
    ///     r#"SELECT "image" FROM "glyph" WHERE "glyph"."aspect" IN (3, 4) AND ("glyph"."image" LIKE 'A%' OR "glyph"."image" LIKE 'B%')"#
    /// );
    /// ```
    ///
    /// Using macro
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Glyph::Image)
    ///     .from(Glyph::Table)
    ///     .cond_where(
    ///         all![
    ///             Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]),
    ///             any![
    ///                 Expr::tbl(Glyph::Table, Glyph::Image).like(&"A%"),
    ///                 Expr::tbl(Glyph::Table, Glyph::Image).like(&"B%")
    ///             ]
    ///         ])
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(),
    ///     r#"SELECT "image" FROM "glyph" WHERE "glyph"."aspect" IN (3, 4) AND ("glyph"."image" LIKE 'A%' OR "glyph"."image" LIKE 'B%')"#
    /// );
    /// ```
    /// Calling multiple times
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// assert_eq!(
    ///     Query::select()
    ///         .cond_where(Cond::all().add(Expr::col(Glyph::Id).eq(&1)))
    ///         .cond_where(
    ///             Cond::any()
    ///                 .add(Expr::col(Glyph::Id).eq(&2))
    ///                 .add(Expr::col(Glyph::Id).eq(&3)),
    ///         )
    ///         .to_owned()
    ///         .to_string(),
    ///     r#"SELECT WHERE "id" = 1 AND ("id" = 2 OR "id" = 3)"#
    /// );
    /// ```
    /// Calling multiple times
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// assert_eq!(
    ///     Query::select()
    ///         .cond_where(
    ///             Cond::any()
    ///                 .add(Expr::col(Glyph::Id).eq(&1))
    ///                 .add(Expr::col(Glyph::Id).eq(&2)),
    ///         )
    ///         .cond_where(Expr::col(Glyph::Id).eq(&3))
    ///         .cond_where(Expr::col(Glyph::Id).eq(&4))
    ///         .to_owned()
    ///         .to_string(),
    ///     r#"SELECT WHERE "id" = 1 OR "id" = 2 OR "id" = 3 OR "id" = 4"#
    /// );
    /// ```
    fn cond_where<C>(&mut self, condition: C) -> &mut Self
    where
        C: IntoCondition<'a, DB>;
}

impl<'a, DB> IntoCondition<'a, DB> for SimpleExpr<'a, DB> {
    fn into_condition(self) -> Condition<'a, DB> {
        Condition::all().add(self)
    }
}

impl<'a, DB> IntoCondition<'a, DB> for Condition<'a, DB> {
    fn into_condition(self) -> Condition<'a, DB> {
        self
    }
}

impl<'a, DB> Default for ConditionHolderContents<'a, DB> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<'a, DB> Default for ConditionHolder<'a, DB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, DB> ConditionHolder<'a, DB> {
    pub fn new() -> Self {
        Self {
            contents: ConditionHolderContents::Empty,
        }
    }

    pub fn new_with_condition(condition: Condition<'a, DB>) -> Self {
        let mut slf = Self::new();
        slf.add_condition(condition);
        slf
    }

    pub fn is_empty(&self) -> bool {
        match &self.contents {
            ConditionHolderContents::Empty => true,
            ConditionHolderContents::Chain(c) => c.is_empty(),
            ConditionHolderContents::Condition(c) => c.conditions.is_empty(),
        }
    }

    pub fn is_one(&self) -> bool {
        match &self.contents {
            ConditionHolderContents::Empty => true,
            ConditionHolderContents::Chain(c) => c.len() == 1,
            ConditionHolderContents::Condition(c) => c.conditions.len() == 1,
        }
    }

    pub fn add_and_or(&mut self, condition: LogicalChainOper<'a, DB>) {
        match &mut self.contents {
            ConditionHolderContents::Empty => {
                self.contents = ConditionHolderContents::Chain(vec![condition])
            }
            ConditionHolderContents::Chain(c) => c.push(condition),
            ConditionHolderContents::Condition(_) => {
                panic!("Cannot mix `and_where`/`or_where` and `cond_where` in statements")
            }
        }
    }

    pub fn add_condition(&mut self, condition: Condition<'a, DB>) {
        match std::mem::take(&mut self.contents) {
            ConditionHolderContents::Empty => {
                self.contents = ConditionHolderContents::Condition(condition);
            }
            ConditionHolderContents::Condition(current) => {
                self.contents = ConditionHolderContents::Condition(current.add(condition));
            }
            ConditionHolderContents::Chain(_) => {
                panic!("Cannot mix `and_where`/`or_where` and `cond_where` in statements")
            }
        }
    }
}
