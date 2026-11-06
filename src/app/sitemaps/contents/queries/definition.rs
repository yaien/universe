macro_rules! query {
    ($({
        ident $e:ident
        label $label:literal
        call $call:ident
        required_args $($arg:literal),*
        args $({
            name $arg_name:literal
            label $arg_label:literal
        }),*
    }),+) => {

        use crate::app::auth::{Organization, User};
        use crate::app::{App, AppResult};
        use minijinja::Value;
        use serde::{Deserialize, Serialize};
        use sqlx::prelude::Type;
        use strum::IntoEnumIterator;
        use strum_macros::{Display, EnumIter};
        use actix_session::Session;
        use std::collections::HashMap;

        #[derive(Serialize)]
        pub struct QueryDescriptor {
            pub name: String,
            pub label: String,
            pub args: &'static [QueryArgs],
        }

        #[derive(Serialize)]
        pub struct QueryArgs {
            pub name: &'static str,
            pub label: &'static str,
        }

        pub struct QueryContext<'a> {
            pub app: &'a App,
            pub org: &'a Organization,
            pub user: &'a Option<User>,
            pub session: &'a Session,
            pub args: &'a HashMap<String, String>,
            pub inline: bool,
        }

        #[derive(EnumIter, Display, Debug, Serialize, Deserialize, Type)]
        pub enum Query {
            $($e),*
        }

        impl Query {
            pub fn get_label(&self) -> &str {
                match self {
                    $(Self::$e => $label),*
                }
            }

            pub fn get_args(&self) -> &'static [QueryArgs] {
                match self {
                    $(Self::$e => &[$(QueryArgs { name: $arg_name, label: $arg_label }),*]),*
                }
            }


            pub fn get_required_args(&self) -> &[&str] {
                match self {
                    $(Self::$e => &[$($arg),*]),*
                }
            }

            pub fn descriptors() -> Vec<QueryDescriptor> {
                Self::iter().map(|q| q.descriptor()).collect()
            }

            pub fn descriptor(&self) -> QueryDescriptor {
                QueryDescriptor {
                    name: format!("{:?}", self),
                    label: self.get_label().to_string(),
                    args: self.get_args(),
                }
            }

            pub async fn call(&self, ctx: &QueryContext<'_>) -> AppResult<Value> {
                match self {
                    $(Self::$e => $call(ctx).await),*
                }
            }


        }

    };
}

pub(crate) use query;
