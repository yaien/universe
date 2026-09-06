use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::web::{Data, Form, Path, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, patch, post, put};
use maud::{Markup, html};
use serde::Deserialize;

use crate::app::store::UpdatePresentationArgs;
use crate::app::{App, Organization, Role};
use crate::infra::Id;
use crate::web::dashboard::views;
use crate::web::dashboard::views::layout::{Content, Variant, toast};
use crate::web::errors::WebError;

#[derive(Deserialize, Default)]
pub struct ProductsQuery {
    pub fragment: Option<String>,
}

#[get("/products")]
pub async fn get_products(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    query: Query<ProductsQuery>,
    req: HttpRequest,
) -> Result<Markup, WebError> {
    let mut query = query.into_inner();
    if !req.headers().contains_key("HX-Request") {
        query = ProductsQuery::default();
    }

    match query.fragment.as_deref() {
        Some("create") => Ok(views::products::create_modal()),

        _ => {
            let products = app.store.products.get_by_organization_id(&org.id).await?;

            Ok(views::layout::layout(&Content {
                title: "Productos",
                path: req.path(),
                org: &org,
                role: &role,
                content: views::products::product_list(products),
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct CreateProductForm {
    pub name: String,
}

#[post("/products")]
pub async fn create_product(
    app: Data<App>,
    org: ReqData<Organization>,
    form: Form<CreateProductForm>,
) -> Result<impl Responder, WebError> {
    let product_id = app.store.products.create(&org.id, &form.name).await?;
    let product_url = format!("/dashboard/products/{}", product_id);
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Location", product_url))
        .finish())
}

#[derive(Deserialize)]
pub struct ProductDetailQuery {
    pub presentation_id: Option<Id>,
    pub content_id: Option<Id>,
    pub fragment: Option<String>,
}

#[get("/products/{id}")]
pub async fn get_product(
    app: Data<App>,
    org: ReqData<Organization>,
    role: ReqData<Role>,
    req: HttpRequest,
    product_id: Path<Id>,
    query: Query<ProductDetailQuery>,
) -> Result<Markup, WebError> {
    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = query
        .presentation_id
        .and_then(|id| product.presentations.iter().find(|p| p.id == id))
        .or(product.presentations.first());

    let content = query
        .content_id
        .and_then(|id| presentation.map(|p| p.contents.iter().find(|c| c.id == id)))
        .flatten();

    match query.fragment.as_deref() {
        Some("pictures") => Ok(html! {
            (views::products::pictures(&product, &presentation, &content))
        }),

        Some("presentations") => Ok(html! {
            (views::products::presentations(&product, &presentation))
            (views::products::pictures_partial(&product, &presentation, &None))
        }),

        Some("delete") => Ok(views::products::delete_modal(&product)),

        _ => Ok(views::layout::layout(&Content {
            title: "Product Details",
            path: req.path(),
            org: &org,
            role: &role,
            content: views::products::product_detail(&product, &presentation),
        })),
    }
}

#[derive(Deserialize)]
pub struct UpdateProductFrom {
    name: String,
    published: bool,
}

#[put("/products/{id}")]
pub async fn update_product(
    app: Data<App>,
    org: ReqData<Organization>,
    product_id: Path<Id>,
    form: Form<UpdateProductFrom>,
) -> Result<Markup, WebError> {
    app.store
        .products
        .update(&org.id, &product_id, &form.name, &form.published)
        .await?;
    Ok(toast(
        "producto actualizado correctamente",
        Variant::Primary,
    ))
}

#[delete("/products/{id}")]
pub async fn delete_product(
    app: Data<App>,
    org: ReqData<Organization>,
    product_id: Path<Id>,
) -> Result<HttpResponse, WebError> {
    app.store.products.delete(&org.id, &product_id).await?;

    Ok(HttpResponse::Ok()
        .insert_header(("HX-Location", "/dashboard/products"))
        .insert_header(("HX-Replace-Url", "/dashboard/products"))
        .finish())
}

#[post("/products/{id}/presentations")]
pub async fn create_presentation(
    app: Data<App>,
    org: ReqData<Organization>,
    product_id: Path<Id>,
) -> Result<Markup, WebError> {
    let presentation = app.store.presentations.create(&org.id, &product_id).await?;
    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;
    Ok(html! {
        (views::products::presentations(&product, &Some(&presentation)))
        (views::products::pictures_partial(&product, &Some(&presentation), &None))
    })
}

#[derive(Deserialize)]
pub struct UpdatePresentationForm {
    name: String,
    quantity: i64,
    price: i64,
}

#[put("/products/{id}/presentations/{pid}")]
pub async fn update_presentation(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    form: Form<UpdatePresentationForm>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id) = path.into_inner();

    app.store
        .presentations
        .update(UpdatePresentationArgs {
            organization_id: &org.id,
            product_id: &product_id,
            presentation_id: &presentation_id,
            name: &form.name,
            quantity: &form.quantity,
            price: &form.price,
        })
        .await?;

    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = product
        .presentations
        .iter()
        .find(|p| p.id == presentation_id);

    Ok(html! {
        (views::products::presentations(&product, &presentation))
        (toast("presentación actualizada", Variant::Primary))
    })
}

#[delete("/products/{id}/presentations/{pid}")]
pub async fn delete_presentation(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id) = path.into_inner();

    app.store
        .presentations
        .delete(&org.id, &product_id, &presentation_id)
        .await?;

    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = product.presentations.first();

    Ok(html! {
        (views::products::presentations(&product, &presentation))
        (views::products::pictures_partial(&product, &presentation, &None))
    })
}

#[derive(Deserialize)]
pub struct SortPresentationForm {
    number: i64,
}

#[patch("/products/{id}/presentations/{pid}/number")]
pub async fn sort_presentation(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    form: Form<SortPresentationForm>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id) = path.into_inner();

    app.store
        .presentations
        .sort(&org.id, &product_id, &presentation_id, &form.number)
        .await?;

    Ok(html!())
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    pub files: Vec<TempFile>,
}

#[post("/products/{id}/presentations/{pid}/contents")]
pub async fn upload_content(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id)>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id) = path.into_inner();

    app.store
        .contents
        .upload_many(&org.id, &product_id, &presentation_id, form.files)
        .await?;

    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = product
        .presentations
        .iter()
        .find(|p| p.id == presentation_id);

    let content = presentation.and_then(|p| p.contents.last());

    Ok(views::products::pictures(&product, &presentation, &content))
}

#[delete("/products/{id}/presentations/{pid}/contents/{cid}")]
pub async fn delete_content(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id, Id)>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id, content_id) = path.into_inner();

    app.store
        .contents
        .delete(&org.id, &product_id, &presentation_id, &content_id)
        .await?;

    let product = app
        .store
        .products
        .get_one_by_organization_id_and_id(&org.id, &product_id)
        .await?;

    let presentation = product
        .presentations
        .iter()
        .find(|p| p.id == presentation_id);

    Ok(views::products::pictures(&product, &presentation, &None))
}

#[derive(Deserialize)]
pub struct SortContentForm {
    pub number: i64,
}

#[patch("/products/{id}/presentations/{pid}/contents/{cid}/number")]
pub async fn sort_content(
    app: Data<App>,
    org: ReqData<Organization>,
    path: Path<(Id, Id, Id)>,
    form: Form<SortContentForm>,
) -> Result<Markup, WebError> {
    let (product_id, presentation_id, content_id) = path.into_inner();

    app.store
        .contents
        .sort(
            &org.id,
            &product_id,
            &presentation_id,
            &content_id,
            &form.number,
        )
        .await?;

    Ok(html!())
}
