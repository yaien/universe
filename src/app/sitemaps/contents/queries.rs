mod definition;
mod store_queries;

use store_queries::*;

definition::query!(
    {
        ident GetProducts
        label "Obtener Productos"
        call get_products
        required_args
        args
    },
    {
        ident GetProduct
        label "Obtener Producto"
        call get_product
        required_args "slug"
        args { name "slug" label "slug" }
    },
    {
        ident GetCart
        label "Obtener Carrito de Compras"
        call get_cart
        required_args
        args
    }
);
