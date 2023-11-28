use shared::User;

use crate::{Field, Filter, FilterKind, JoinFilters};

fn get_field(user: &User, field: Field) -> &str {
    match field {
        Field::Email => &user.email,
        Field::Gender => &user.gender,
        Field::PhoneNumber => &user.phone_number,
        Field::LocationStreet => &user.location.street,
        Field::LocationCity => &user.location.city,
        Field::LocationState => &user.location.state,
        Field::Username => &user.username,
        Field::Password => &user.password,
        Field::FirstName => &user.first_name,
        Field::LastName => &user.last_name,
        Field::Title => &user.title,
        Field::Picture => &user.picture,
    }
}

pub fn run_filter(user: &User, filter: &Filter) -> bool {
    let field = get_field(user, filter.field);
    match filter.kind {
        FilterKind::StrContains => field.contains(&filter.value),
        FilterKind::StrEquals => field == &filter.value,
        FilterKind::StrStartsWith => field.starts_with(&filter.value),
        FilterKind::StrEndsWith => field.ends_with(&filter.value),
    }
}

pub fn run_join_filters(user: &User, join_filters: &JoinFilters) -> bool {
    match join_filters {
        JoinFilters::Filter(filter) => run_filter(user, filter),
        JoinFilters::And(left, right) => {
            run_join_filters(user, left) && run_join_filters(user, right)
        }
        JoinFilters::Or(left, right) => {
            run_join_filters(user, left) || run_join_filters(user, right)
        }
    }
}

pub fn filter_vec_with_filters(arr: &[User], filters: &JoinFilters) -> Vec<User> {
    arr.iter()
        .filter(|user| run_join_filters(user, filters))
        .cloned()
        .collect()
}
