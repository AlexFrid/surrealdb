use crate::ctx::Context;
use crate::dbs::Options;
use crate::dbs::Transaction;
use crate::err::Error;
use crate::iam::{Action, ResourceKind};
use crate::sql::base::Base;
use crate::sql::comment::shouldbespace;
use crate::sql::error::IResult;
use crate::sql::ident::{ident, Ident};
use crate::sql::value::Value;
use derive::Store;
use nom::bytes::complete::tag_no_case;
use revision::revisioned;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Store, Hash)]
#[revisioned(revision = 1)]
pub struct RemoveScopeStatement {
	pub name: Ident,
}

impl RemoveScopeStatement {
	/// Process this type returning a computed simple Value
	pub(crate) async fn compute(
		&self,
		_ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Value, Error> {
		// Allowed to run?
		opt.is_allowed(Action::Edit, ResourceKind::Scope, &Base::Db)?;
		// Claim transaction
		let mut run = txn.lock().await;
		// Clear the cache
		run.clear_cache();
		// Delete the definition
		let key = crate::key::database::sc::new(opt.ns(), opt.db(), &self.name);
		run.del(key).await?;
		// Remove the resource data
		let key = crate::key::scope::all::new(opt.ns(), opt.db(), &self.name);
		run.delp(key, u32::MAX).await?;
		// Ok all good
		Ok(Value::None)
	}
}

impl Display for RemoveScopeStatement {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "REMOVE SCOPE {}", self.name)
	}
}

pub fn scope(i: &str) -> IResult<&str, RemoveScopeStatement> {
	let (i, _) = tag_no_case("REMOVE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, _) = tag_no_case("SCOPE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, name) = ident(i)?;
	Ok((
		i,
		RemoveScopeStatement {
			name,
		},
	))
}
