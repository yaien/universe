use std::collections::HashMap;

use slug::slugify;
use sqlx::QueryBuilder;
use sqlx::prelude::FromRow;

use crate::app::AppError;
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct Product {
    pub id: Id,
    pub slug: String,
    pub name: String,
    pub published: bool,

    #[sqlx(skip)]
    pub presentations: Vec<Presentation>,
}

#[derive(FromRow)]
pub struct Presentation {
    pub id: Id,
    pub product_id: Id,
    pub name: String,
    pub quantity: i64,
    pub price: f64,
    pub number: i64,

    #[sqlx(skip)]
    pub contents: Vec<Content>,
}

#[derive(FromRow)]
pub struct Content {
    pub id: Id,
    pub number: i64,
    pub presentation_id: Id,
    pub file_id: Id,
    pub file_preset: String,
}

pub struct Products {
    db: DbPool,
}

impl Products {
    pub const MAX_CONTENTS_PER_PRESENTATION: usize = 5;

    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, organization_id: &Id, name: &str) -> Result<Id, AppError> {
        if name.is_empty() {
            return Err(AppError::Message(
                "el nombre de el producto no puede estar vacio".into(),
            ));
        }

        let slug = slugify(name);

        let exists: bool = sqlx::query_scalar(
            "select count(*) > 0 from products where organization_id = $1 and slug = $2",
        )
        .bind(organization_id)
        .bind(&slug)
        .fetch_one(&self.db)
        .await?;

        if exists {
            return Err(AppError::Message(
                "el nombre de el producto ya existe".into(),
            ));
        }

        let inserted_id = sqlx::query("insert into products(organization_id, name, slug)")
            .bind(organization_id)
            .bind(name)
            .bind(&slug)
            .execute(&self.db)
            .await
            .map(|r| r.last_insert_rowid())?;

        Ok(inserted_id)
    }

    pub async fn get_one_by_organization_id_and_id(
        &self,
        organization_id: &Id,
        product_id: &Id,
    ) -> Result<Product, AppError> {
        let mut product = sqlx::query_as::<_, Product>(
            "select * from products where organization_id = $1 and id = $2",
        )
        .bind(organization_id)
        .bind(product_id)
        .fetch_one(&self.db)
        .await?;

        product.presentations = sqlx::query_as::<_, Presentation>(
            "select * from presentations where product_id = $1 order by number",
        )
        .bind(&product.id)
        .fetch_all(&self.db)
        .await?;

        let contents = QueryBuilder::new(
                 "select c.id, c.presentation_id, c.number, c.file_id, f.preset from contents c join files f on f.id = c.file_id where c.presentation_id in ",
            ).push_tuples(&product.presentations, |mut b, p| {
                b.push_bind(p.id);
            })
            .push(" order by number")
            .build_query_as::<Content>()
            .fetch_all(&self.db).await?;

        let mut presentations_map = {
            let mut map = HashMap::new();
            for presentation in product.presentations.iter_mut() {
                map.insert(presentation.id, presentation);
            }
            map
        };

        for content in contents {
            if let Some(presentation) = presentations_map.get_mut(&content.presentation_id) {
                presentation.contents.push(content);
            }
        }

        Ok(product)
    }

    pub async fn get_by_organization_id(
        &self,
        organization_id: &Id,
    ) -> Result<Vec<Product>, AppError> {
        let mut products = sqlx::query_as::<_, Product>(
            "select * from products where organization_id = $1 and deleted_at is null",
        )
        .bind(organization_id)
        .fetch_all(&self.db)
        .await?;

        let mut presentations =
            QueryBuilder::new("select * from presentations where product_id in ")
                .push_tuples(&products, |mut b, p| {
                    b.push_bind(p.id);
                })
                .build_query_as::<Presentation>()
                .fetch_all(&self.db)
                .await?;

        let contents = QueryBuilder::new("select c.id, c.presentation_id, c.file_id, f.preset from contents c join files f on f.id = c.file_id where c.presentation_id in ")
            .push_tuples(&presentations, |mut b, p| {
                b.push_bind(p.id);
            })
            .build_query_as::<Content>()
            .fetch_all(&self.db)
            .await?;

        let mut presentations_map: HashMap<Id, &mut Presentation> =
            presentations.iter_mut().map(|p| (p.id, p)).collect();

        for content in contents {
            if let Some(presentation) = presentations_map.get_mut(&content.presentation_id) {
                presentation.contents.push(content);
            }
        }

        let mut products_map: HashMap<Id, &mut Product> =
            products.iter_mut().map(|p| (p.id, p)).collect();

        for presentation in presentations {
            if let Some(product) = products_map.get_mut(&presentation.product_id) {
                product.presentations.push(presentation);
            }
        }

        Ok(products)
    }
}
