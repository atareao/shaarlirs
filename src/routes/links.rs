use actix_web::{get, post, put, delete, web, error::{ErrorNotFound,
    ErrorConflict, ErrorUnauthorized}, Error, HttpResponse};

use crate::models::{link::{LinkWithTags, Link}, metatag::Metatag};


/*
pub async fn create(pool: &web::Data<PgPool>, new_link_with_tags: web::Json<LinkWithTags>) -> Result<HttpResponse, Error>{

    let link = Link::create(&pool, url)
            let name = category.into_inner().name;
            Category::new(pool, &name, user_id)
                .await
                .map(|item| HttpResponse::Ok().json(item))
                .map_err(|e| ErrorConflict(e))
        },
        Err(e) => Err(ErrorUnauthorized(e)),
    }
}

*/
