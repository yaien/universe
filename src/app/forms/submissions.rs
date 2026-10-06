use std::collections::HashMap;

use sqlx::QueryBuilder;
use sqlx::prelude::FromRow;

use crate::app::AppResult;
use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct FormSubmissionAnswer {
    pub field_id: Id,
    pub value: String,
    pub submission_id: Id,
}

#[derive(FromRow)]
pub struct FormSubmission {
    pub id: Id,

    #[sqlx(skip)]
    pub answers: Vec<FormSubmissionAnswer>,
}

pub struct FormSubmissions {
    pool: DbPool,
}

impl FormSubmissions {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find(
        &self,
        form_id: &Id,
        search: &Option<String>,
        offset: &Option<u32>,
        limit: &Option<u32>,
    ) -> AppResult<Vec<FormSubmission>> {
        let mut query = QueryBuilder::new("select * from form_submissions where form_id = ");

        query.push_bind(form_id);

        if let Some(search) = search {
            query
                .push(" and exists (select 1 from form_submission_answers where submission_id = form_submissions.id and value like ")
                .push_bind(format!("%{}%", search))
                .push(")");
        }

        query.push(" order by id desc");

        query
            .push(" limit ")
            .push_bind(limit.filter(|l| *l <= 20).unwrap_or(10));

        query.push(" offset ").push_bind(offset.unwrap_or(0));

        let mut submissions = query
            .build_query_as::<FormSubmission>()
            .fetch_all(&self.pool)
            .await?;

        let mut query =
            QueryBuilder::new("select * from form_submission_answers where submission_id in ");

        query.push_tuples(&submissions, |mut t, submission| {
            t.push_bind(&submission.id);
        });

        let answers = query
            .build_query_as::<FormSubmissionAnswer>()
            .fetch_all(&self.pool)
            .await?;

        let mut submissions_map = submissions
            .iter_mut()
            .map(|s| (s.id, s))
            .collect::<HashMap<_, _>>();

        for answer in answers {
            if let Some(submission) = submissions_map.get_mut(&answer.submission_id) {
                (*submission).answers.push(answer);
            }
        }

        Ok(submissions)
    }
}
