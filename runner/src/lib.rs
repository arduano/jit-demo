use shared::User;

pub mod interpreted;
pub mod jit;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum FilterKind {
    StrContains,
    StrEquals,
    StrStartsWith,
    StrEndsWith,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Field {
    Email,
    Gender,
    PhoneNumber,
    LocationStreet,
    LocationCity,
    LocationState,
    Username,
    Password,
    FirstName,
    LastName,
    Title,
    Picture,
}

#[derive(Debug, Clone)]
pub struct Filter {
    field: Field,
    kind: FilterKind,
    value: String,
}

#[derive(Debug, Clone)]
pub enum JoinFilters {
    Filter(Filter),
    And(Box<JoinFilters>, Box<JoinFilters>),
    Or(Box<JoinFilters>, Box<JoinFilters>),
}

pub fn read_data() -> Vec<User> {
    let contents = include_str!("../../data.json");
    serde_json::from_str(&contents).unwrap()
}

pub fn build_complex_filter() -> JoinFilters {
    // Very arbitrary complex filters

    // The burner filter is designed to fail on all users with many "or" statements, wasting a bunch of cpu time intentionally.
    // It demonstrates that the JIT has a much better performance improvement, partly because it optimizes a lot of this away.
    // If you don't use the burner filter, the JIT will still be faster than interpreted, just not as much faster.
    let burner_filter = JoinFilters::Filter(Filter {
        field: Field::FirstName,
        kind: FilterKind::StrStartsWith,
        value: "a long value".to_string(),
    });
    let burner_filter = JoinFilters::Or(
        Box::new(burner_filter.clone()),
        Box::new(burner_filter.clone()),
    );
    let burner_filter = JoinFilters::Or(
        Box::new(burner_filter.clone()),
        Box::new(burner_filter.clone()),
    );
    let burner_filter = JoinFilters::Or(
        Box::new(burner_filter.clone()),
        Box::new(burner_filter.clone()),
    );
    let burner_filter = JoinFilters::Or(
        Box::new(burner_filter.clone()),
        Box::new(burner_filter.clone()),
    );
    let burner_filter = JoinFilters::Or(
        Box::new(burner_filter.clone()),
        Box::new(burner_filter.clone()),
    );

    let complex_filter_1 = JoinFilters::And(
        Box::new(JoinFilters::Or(
            Box::new(JoinFilters::Filter(Filter {
                field: Field::Email,
                kind: FilterKind::StrContains,
                value: "example.com".to_string(),
            })),
            Box::new(JoinFilters::Filter(Filter {
                field: Field::LocationCity,
                kind: FilterKind::StrEquals,
                value: "New York".to_string(),
            })),
        )),
        Box::new(JoinFilters::Filter(Filter {
            field: Field::Gender,
            kind: FilterKind::StrEquals,
            value: "female".to_string(),
        })),
    );

    let complex_filter_2 = JoinFilters::Or(
        Box::new(JoinFilters::And(
            Box::new(JoinFilters::Filter(Filter {
                field: Field::Username,
                kind: FilterKind::StrStartsWith,
                value: "user_".to_string(),
            })),
            Box::new(JoinFilters::Filter(Filter {
                field: Field::LocationState,
                kind: FilterKind::StrEndsWith,
                value: "shire".to_string(),
            })),
        )),
        Box::new(JoinFilters::Filter(Filter {
            field: Field::PhoneNumber,
            kind: FilterKind::StrContains,
            value: "+123".to_string(),
        })),
    );

    let complex_filter_3 = JoinFilters::And(
        Box::new(JoinFilters::Filter(Filter {
            field: Field::FirstName,
            kind: FilterKind::StrEquals,
            value: "John".to_string(),
        })),
        Box::new(JoinFilters::Or(
            Box::new(JoinFilters::Filter(Filter {
                field: Field::LastName,
                kind: FilterKind::StrEquals,
                value: "Doe".to_string(),
            })),
            Box::new(JoinFilters::And(
                Box::new(JoinFilters::Filter(Filter {
                    field: Field::LocationCity,
                    kind: FilterKind::StrEquals,
                    value: "London".to_string(),
                })),
                Box::new(JoinFilters::Filter(Filter {
                    field: Field::Title,
                    kind: FilterKind::StrEquals,
                    value: "Dr".to_string(),
                })),
            )),
        )),
    );

    let commplex_joined = JoinFilters::Or(
        Box::new(JoinFilters::Or(
            Box::new(complex_filter_1),
            Box::new(complex_filter_2),
        )),
        Box::new(complex_filter_3),
    );

    JoinFilters::Or(Box::new(burner_filter), Box::new(commplex_joined))
}
